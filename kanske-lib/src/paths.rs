use std::{env, path::PathBuf};

use crate::AppResult;

pub fn pid_file_path() -> AppResult<PathBuf> {
    let dir = match env::var_os("XDG_RUNTIME_DIR") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("/tmp"),
    };
    Ok(dir.join("kanske.pid"))
}
