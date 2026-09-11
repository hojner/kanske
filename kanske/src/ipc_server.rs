//! Control socket for `kanskectl status` / `kanskectl switch <profile>`.
//!
//! One request/response per connection: accept, read a single line, handle it, write a
//! single line back, and let the stream close. See `kanske_lib::ipc` for the wire format.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

use calloop::generic::Generic;
use calloop::{Interest, LoopHandle, Mode, PostAction};
use tracing::warn;

use kanske_lib::{
    AppResult,
    applier::apply_named_profile,
    error::KanskeError,
    ipc::{Request, Response},
    paths::socket_path,
};

use crate::exec::run_exec_commands;
use crate::state::KanskeState;

/// Timeout for reading a request from / writing a response to a connected client, so a
/// stalled or misbehaving client can't block the event loop indefinitely.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Removes the control socket file on drop, mirroring the PID-file cleanup pattern in
/// `main.rs`.
pub struct SocketFailGuard(PathBuf);

impl Drop for SocketFailGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Binds the control socket in non-blocking mode, removing any stale socket file left
/// behind by an unclean previous shutdown.
pub fn create_socket() -> AppResult<(UnixListener, SocketFailGuard)> {
    let path = socket_path()?;
    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    listener.set_nonblocking(true)?;
    Ok((listener, SocketFailGuard(path)))
}

/// Registers the control socket with the event loop. On readiness, accepts and fully
/// handles every pending connection (one request/response each) before yielding.
pub fn register(loop_handle: &LoopHandle<'_, KanskeState>, listener: UnixListener) -> AppResult<()> {
    loop_handle
        .insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            |_readiness, listener, state| {
                loop {
                    match listener.accept() {
                        Ok((stream, _addr)) => handle_connection(stream, state),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            warn!("Failed to accept IPC connection: {}", e);
                            break;
                        }
                    }
                }
                Ok(PostAction::Continue)
            },
        )
        .map_err(|e| KanskeError::CalloopError(e.to_string()))?;
    Ok(())
}

fn handle_connection(mut stream: UnixStream, state: &mut KanskeState) {
    if let Err(e) = stream.set_read_timeout(Some(REQUEST_TIMEOUT)) {
        warn!("Failed to set IPC read timeout: {}", e);
    }
    if let Err(e) = stream.set_write_timeout(Some(REQUEST_TIMEOUT)) {
        warn!("Failed to set IPC write timeout: {}", e);
    }

    let mut line = String::new();
    {
        let mut reader = BufReader::new(&stream);
        match reader.read_line(&mut line) {
            Ok(0) => return, // client disconnected without sending anything
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to read IPC request: {}", e);
                return;
            }
        }
    }

    let response = match Request::from_line(&line) {
        Ok(Request::Status) => handle_status(state),
        Ok(Request::Switch(name)) => handle_switch(state, &name),
        Err(e) => Response::Err(e.to_string()),
    };

    if let Err(e) = writeln!(stream, "{}", response.to_line()) {
        warn!("Failed to write IPC response: {}", e);
    }
}

fn handle_status(state: &KanskeState) -> Response {
    let profile = state.current_profile.as_deref().unwrap_or("none");
    Response::Ok(format!(
        "profile={} heads={}",
        profile,
        state.wayland.heads.len()
    ))
}

fn handle_switch(state: &mut KanskeState, name: &str) -> Response {
    match apply_named_profile(&mut state.wayland, &state.queue_handle, &state.config, name) {
        Ok((applied, config_obj)) => {
            if let Some(applied) = &applied {
                run_exec_commands(&applied.execs);
            }
            if let Some(c) = config_obj {
                c.destroy();
            }
            if let Err(e) = state.connection.flush() {
                warn!("Failed to flush switch request: {}", e);
            }
            // The compositor will emit a new configuration serial acknowledging this
            // switch; suppress the automatic hotplug re-match for that serial change so
            // it doesn't immediately revert this manual switch back to whatever profile
            // matches the (unchanged) physical heads.
            state.manual_switch_pending = true;
            let name = applied.and_then(|p| p.name).unwrap_or_default();
            state.current_profile = Some(name.clone());
            Response::Ok(format!("Switched to profile '{}'", name))
        }
        Err(e) => Response::Err(e.to_string()),
    }
}
