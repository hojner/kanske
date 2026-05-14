use std::{env, path::PathBuf};

use crate::AppResult;

fn runtime_dir() -> PathBuf {
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("/tmp"),
    }
}

pub fn pid_file_path() -> AppResult<PathBuf> {
    Ok(runtime_dir().join("kanske.pid"))
}

pub fn socket_path() -> AppResult<PathBuf> {
    Ok(runtime_dir().join("kanske.sock"))
}
