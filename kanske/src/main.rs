pub mod exec;
pub mod state;

use std::path::PathBuf;
use std::{env, fs, process};

use kanske_lib::{
    AppResult,
    applier::find_and_apply_profile,
    error::KanskeError,
    parser::{ast::Config, config_parser::parse_file},
    wayland_interface::{WaylandState, connect},
};
use tracing::{debug, info, warn};
use wayland_client::{EventQueue, QueueHandle};

use crate::exec::run_exec_commands;
use crate::state::KanskeState;

const DEFAULT_CONFIG: &str = include_str!("../default_config");

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
    let (_connection, mut event_queue, qh) = connect::<KanskeState>()?;

    let mut state = KanskeState {
        wayland: WaylandState {
            manager: None,
            heads: Vec::new(),
            serial: None,
        },
        config: config.clone(),
        queue_handle: qh.clone(),
        last_serial: None,
    };
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    info!("Monitoring for display changes...");

    event_loop(&mut state, &mut event_queue, &qh, &config)
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

fn event_loop(
    state: &mut KanskeState,
    event_queue: &mut EventQueue<KanskeState>,
    queue_handle: &QueueHandle<KanskeState>,
    config: &Config,
) -> AppResult<()> {
    loop {
        event_queue.blocking_dispatch(state)?;

        if state.wayland.serial != state.last_serial {
            info!("Display hotplug detected");
            debug!(previous_serial = ?state.last_serial, new_serial = ?state.wayland.serial, heads = state.wayland.heads.len());
            match find_and_apply_profile(&mut state.wayland, queue_handle, config) {
                Ok(execs) => run_exec_commands(&execs),
                Err(e @ (KanskeError::ManagerNotAvailable | KanskeError::NoSerial)) => {
                    return Err(e);
                }
                Err(e) => {
                    warn!("Profile apply failed: {}", e);
                }
            }
            // Drain events from our own apply (Succeeded/Failed + new Done serial)
            // so we don't re-trigger on our own configuration change.
            event_queue.roundtrip(state)?;
            state.last_serial = state.wayland.serial;
        }
    }
}
