use nix::{errno::Errno, sys::signal, unistd::Pid};
use std::fs;

use clap::{Parser, Subcommand};
use kanske_lib::{
    AppResult,
    error::KanskeError,
    paths::pid_file_path,
    wayland_interface::{WaylandState, connect},
};

#[derive(Parser)]
#[command(name = "kanskectl", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List anything you want!
    List,
    /// Reload when needed.
    Reload,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::List => {
            let _outputs = list_outputs();
        }
        Commands::Reload => match reload() {
            Ok(_) => println!("Kanske config reloaded"),
            Err(e) => println!("Kanske reload failed: {}", e),
        },
    }
}

fn list_outputs() -> AppResult<()> {
    let (_connection, mut event_queue, _qh) = connect::<WaylandState>()?;
    let mut state = WaylandState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    for head in state.heads {
        print!("{}", head);
    }

    Ok(())
}

fn reload() -> AppResult<()> {
    let pid_path = pid_file_path()?;
    let pid_str = fs::read_to_string(&pid_path).map_err(|_| KanskeError::DaemonNotRunning)?;
    let pid: i32 = pid_str
        .trim()
        .parse::<i32>()
        .map_err(|_| KanskeError::InvalidPidFile)?;

    signal::kill(Pid::from_raw(pid), signal::Signal::SIGHUP).map_err(|e| match e {
        Errno::ESRCH => KanskeError::DaemonNotRunning,
        other => KanskeError::SignalFailed(other.to_string()),
    })?;

    println!("Sent SIGHUP to kanske (pid {})", pid);
    Ok(())
}
