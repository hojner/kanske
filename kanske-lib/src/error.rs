use std::num::ParseIntError;

// pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug)]
pub enum KanskeError {
    WaylandConnectError(wayland_client::ConnectError),
    ParsedStringIsEmpty,
    ParsedStringUnexpectedFormat(String),
    ReadIOError(std::io::Error),
    LimitedFunctionError(String),
}

pub type AppResult<T> = std::result::Result<T, KanskeError>;
