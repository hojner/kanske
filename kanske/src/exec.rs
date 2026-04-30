use std::process;

use kanske_lib::parser::ast::ExecDirective;
use tracing::{debug, info, warn};

pub fn run_exec_commands(execs: &[ExecDirective]) {
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
