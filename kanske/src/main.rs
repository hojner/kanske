pub mod exec;
pub mod state;

use calloop::{
    EventLoop,
    signals::{Signal, Signals},
};
use calloop_wayland_source::WaylandSource;
use std::path::PathBuf;
use std::{env, fs, process};

use kanske_lib::{
    AppResult,
    applier::find_and_apply_profile,
    error::KanskeError,
    parser::config_parser::parse_file,
    paths::pid_file_path,
    wayland_interface::{WaylandState, connect},
};
use tracing::{debug, info, warn};
use wayland_client::EventQueue;

use crate::exec::run_exec_commands;
use crate::state::KanskeState;

const DEFAULT_CONFIG: &str = include_str!("../default_config");

struct PidFailGuard(PathBuf);
impl Drop for PidFailGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("kanske=info,kanske_lib=info")
            }),
        )
        .try_init();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let config_file = config_path()?;
    ensure_config(&config_file)?;
    info!(config = %config_file.display(), "Loading config");
    let config = parse_file(config_file)?;
    let (connection, mut event_queue, queue_handle) = connect::<KanskeState>()?;

    let mut state = KanskeState {
        wayland: WaylandState {
            manager: None,
            heads: Vec::new(),
            serial: None,
        },
        config,
        queue_handle,
        connection: connection.clone(),
        last_serial: None,
        reload_pending: false,
    };
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    let mut event_loop: EventLoop<KanskeState> =
        EventLoop::try_new().map_err(|e| KanskeError::CalloopError(e.to_string()))?;
    let signal = event_loop.get_signal();
    let signal_handle = signal.clone();
    let loop_handle = event_loop.handle();

    loop_handle
        .insert_source(
            WaylandSource::new(connection, event_queue),
            move |_event, queue, state| {
                let count = queue.dispatch_pending(state)?;
                if let Err(e) = apply_if_changed(state, queue) {
                    warn!("Fatal apply error, stopping: {}", e);
                    signal.stop();
                }
                Ok(count)
            },
        )
        .map_err(|e| KanskeError::CalloopError(e.to_string()))?;

    let signals = [Signal::SIGHUP, Signal::SIGINT, Signal::SIGTERM];
    loop_handle
        .insert_source(
            Signals::new(&signals).map_err(|e| KanskeError::CalloopError(e.to_string()))?,
            move |signal_event, _meta, state| match signal_event.signal() {
                Signal::SIGINT | Signal::SIGTERM => {
                    info!(signal = ?signal_event.signal(), "Shutdown signal received");
                    signal_handle.stop();
                }
                Signal::SIGHUP => {
                    info!("SIGHUP received, reloading config");
                    if let Err(e) = reload_config(state) {
                        warn!("Config reload failed, keeping previous config: {}", e);
                        return;
                    }
                    let _ = state.connection.display().sync(&state.queue_handle, ());
                    if let Err(e) = state.connection.flush() {
                        warn!("Failed to flush sync request: {}", e);
                    }
                }
                _ => {}
            },
        )
        .map_err(|e| KanskeError::CalloopError(e.to_string()))?;

    let pid_path = pid_file_path()?;
    fs::write(&pid_path, std::process::id().to_string())?;
    let _pid_guard = PidFailGuard(pid_path.clone());

    info!("Monitoring for display changes...");

    event_loop
        .run(None, &mut state, |_| {})
        .map_err(|e| KanskeError::CalloopError(e.to_string()))?;

    info!("Event loop exited");
    Ok(())
}

fn ensure_config(path: &PathBuf) -> AppResult<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_CONFIG)?;
    info!(path = %path.display(), "Created default config file");

    Ok(())
}

fn config_path() -> AppResult<PathBuf> {
    let config_dir = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => {
            let home = env::var_os("HOME").ok_or(KanskeError::NoConfigDir)?;
            PathBuf::from(home).join(".config")
        }
    };

    Ok(config_dir.join("kanske").join("config"))
}

fn apply_if_changed(state: &mut KanskeState, queue: &mut EventQueue<KanskeState>) -> AppResult<()> {
    if state.wayland.serial == state.last_serial || state.wayland.serial.is_none() {
        return Ok(());
    }
    let reason = if std::mem::take(&mut state.reload_pending) {
        "Config reloaded"
    } else {
        "Display hotplug detected"
    };
    info!("{}", reason);
    debug!(previous_serial = ?state.last_serial, new_serial = ?state.wayland.serial, heads = state.wayland.heads.len());

    let config_obj = match find_and_apply_profile(&mut state.wayland, &state.queue_handle, &state.config) {
        Ok((execs, config_obj)) => {
            run_exec_commands(&execs);
            config_obj
        }
        Err(e @ (KanskeError::ManagerNotAvailable | KanskeError::NoSerial)) => {
            return Err(e);
        }
        Err(e) => {
            warn!("Profile apply failed: {}", e);
            None
        }
    };
    queue.roundtrip(state)?;
    if let Some(c) = config_obj {
        c.destroy();
    }
    state.last_serial = state.wayland.serial;

    Ok(())
}

fn reload_config(state: &mut KanskeState) -> AppResult<()> {
    let config_file = config_path()?;
    info!(config = %config_file.display(), "Reloading config");
    state.config = parse_file(config_file)?;
    state.reload_pending = true;
    state.last_serial = None;
    Ok(())
}
