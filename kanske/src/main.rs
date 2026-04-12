use std::{env, fs, path::PathBuf, process};

use kanske_lib::{
    AppResult, KanskeState,
    applier::find_and_apply_profile,
    error::KanskeError,
    parser::{
        ast::{Config, ExecDirective},
        config_parser::parse_file,
    },
};
use tracing::{debug, info, warn};
use wayland_client::{Connection, EventQueue, QueueHandle};

const DEFAULT_CONFIG: &str = include_str!("../default_config");

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("kanske=info,kanske_lib=info")),
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
    let (mut state, mut event_queue, queue_handle) = wayland_setup()?;

    info!("Monitoring for display changes...");

    event_loop(&mut state, &mut event_queue, &queue_handle, &config)
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

fn wayland_setup() -> AppResult<(
    KanskeState,
    EventQueue<KanskeState>,
    QueueHandle<KanskeState>,
)> {
    let wayland_connection = Connection::connect_to_env()?;
    debug!("Wayland connection established");
    let mut event_queue = wayland_connection.new_event_queue();
    let queue_handle: QueueHandle<KanskeState> = event_queue.handle();

    let mut state = KanskeState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };
    wayland_connection.display().get_registry(&queue_handle, ());

    // Decision: Making two roundtrips will give us the initial state of
    // the app. It's not pretty but gets the job done, and seems to be the
    // prefered option for many production apps. For now it's good
    // enough, might be handled in a while !ready loop later.
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;
    debug!(heads = state.heads.len(), serial = ?state.serial, "Initial roundtrip complete");

    Ok((state, event_queue, queue_handle))
}

fn event_loop(
    state: &mut KanskeState,
    event_queue: &mut EventQueue<KanskeState>,
    queue_handle: &QueueHandle<KanskeState>,
    config: &Config,
) -> AppResult<()> {
    let mut last_serial = state.serial;

    loop {
        event_queue.blocking_dispatch(state)?;
        if state.serial != last_serial {
            info!("Display hotplug detected");
            debug!(previous_serial = ?last_serial, new_serial = ?state.serial, heads = state.heads.len());
            match find_and_apply_profile(state, queue_handle, config) {
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
            last_serial = state.serial;
        }
    }
}

fn run_exec_commands(execs: &[ExecDirective]) {
    for exec in execs {
        info!(command = %exec.command, "Running exec command");
        match process::Command::new("sh")
            .arg("-c")
            .arg(&exec.command)
            .spawn()
        {
            Ok(child) => {
                debug!(pid = child.id(), command = %exec.command, "Spawned exec process");
            }
            Err(e) => {
                warn!(command = %exec.command, error = %e, "Failed to spawn exec command");
            }
        }
    }
}
