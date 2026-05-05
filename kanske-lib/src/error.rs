use crate::parser::token::TokenPosition;

#[derive(Debug)]
pub enum KanskeError {
    ConfigError {
        file: String,
        source: ConfigParseError,
    },
    ReadIOError(std::io::Error),
    WaylandConnectError(wayland_client::ConnectError),
    WaylandDispatchError(wayland_client::DispatchError),
    ManagerNotAvailable,
    NoSerial,
    HeadNotFound {
        name: String,
    },
    CalloopError(String),
    NoConfigDir,
    DaemonNotRunning,
    InvalidPidFile,
    SignalFailed(String),
}

#[derive(Debug)]
pub enum ConfigParseError {
    ParsedStringIsEmpty,
    ParsedStringUnexpectedFormat(String),
    UnexpectedToken {
        expected: String,
        found: String,
        position: TokenPosition,
    },
    UnterminatedString {
        line: usize,
    },
    InvalidNumber {
        value: String,
        position: usize,
    },
    InvalidResolution {
        value: String,
        reason: String,
        position: TokenPosition,
    },
    InvalidPosition {
        value: String,
        reason: String,
        position: TokenPosition,
    },
    InvalidTransform {
        value: String,
        position: TokenPosition,
    },
    InvalidAdaptiveSync {
        value: String,
        position: TokenPosition,
    },
    MissingProfileName {
        position: TokenPosition,
    },
    IncludeError {
        path: String,
        reason: String,
    },
    IncludeDepthExceeded {
        path: String,
    },
    UnexpectedCharacter {
        character: char,
        position: usize,
        line: usize,
    },
    TokenNotAvailable,
}

impl From<wayland_client::DispatchError> for KanskeError {
    fn from(err: wayland_client::DispatchError) -> Self {
        KanskeError::WaylandDispatchError(err)
    }
}

impl From<std::io::Error> for KanskeError {
    fn from(err: std::io::Error) -> Self {
        KanskeError::ReadIOError(err)
    }
}

impl From<wayland_client::ConnectError> for KanskeError {
    fn from(err: wayland_client::ConnectError) -> Self {
        KanskeError::WaylandConnectError(err)
    }
}

impl ConfigParseError {
    pub fn into_config_error(self, file: String) -> KanskeError {
        KanskeError::ConfigError { file, source: self }
    }
}

impl std::fmt::Display for KanskeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KanskeError::ConfigError { file, source } => {
                write!(f, "Config file error. File: {}. Error: {}", file, source)
            }
            KanskeError::ReadIOError(e) => write!(f, "Error reading IO: {}", e),
            KanskeError::WaylandConnectError(e) => write!(f, "Wayland connection error: {}", e),
            KanskeError::WaylandDispatchError(e) => write!(f, "Dispatch error: {}", e),
            KanskeError::ManagerNotAvailable => {
                write!(f, "Cannot get Display Manager from Wayland")
            }
            KanskeError::NoSerial => {
                write!(f, "No configuration serial received from compositor")
            }
            KanskeError::HeadNotFound { name } => {
                write!(f, "Output head not found: {}", name)
            }
            KanskeError::CalloopError(s) => write!(f, "Calloop error: {}", s),
            KanskeError::NoConfigDir => write!(
                f,
                "Could not determine config directory: neither XDG_CONFIG_HOME nor HOME is set"
            ),
            KanskeError::DaemonNotRunning => {
                write!(f, "The Kanske daemon does not seem to be running")
            }
            KanskeError::InvalidPidFile => write!(f, "The Kanske Pid file is invalid"),
            KanskeError::SignalFailed(s) => write!(f, "kanskectl failed to send signal: {}", s),
        }
    }
}

impl std::fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigParseError::ParsedStringIsEmpty => write!(f, "Parsed string is empty"),
            ConfigParseError::ParsedStringUnexpectedFormat(msg) => {
                write!(f, "Unexpected format: {}", msg)
            }
            ConfigParseError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Unexpected token at {}: expected {}, found {}",
                    position, expected, found
                )
            }
            ConfigParseError::UnterminatedString { line } => {
                write!(
                    f,
                    "Unterminated string in config, starting at line {}",
                    line
                )
            }
            ConfigParseError::InvalidNumber { value, position } => {
                write!(f, "Invalid number '{}' at position {}", value, position)
            }
            ConfigParseError::InvalidResolution { value, reason, position } => {
                write!(f, "Invalid resolution '{}' at {}: {}", value, position, reason)
            }
            ConfigParseError::InvalidPosition { value, reason, position } => {
                write!(f, "Invalid position '{}' at {}: {}", value, position, reason)
            }
            ConfigParseError::InvalidTransform { value, position } => {
                write!(
                    f,
                    "Invalid transform '{}' at {}: must be one of: normal, 90, 180, 270, flipped, flipped-90, flipped-180, flipped-270",
                    value, position
                )
            }
            ConfigParseError::InvalidAdaptiveSync { value, position } => {
                write!(
                    f,
                    "Invalid adaptive_sync value '{}' at {}: must be 'on' or 'off'",
                    value, position
                )
            }
            ConfigParseError::MissingProfileName { position } => {
                write!(f, "Missing profile name at position {}", position)
            }
            ConfigParseError::IncludeError { path, reason } => {
                write!(f, "Include error for '{}': {}", path, reason)
            }
            ConfigParseError::IncludeDepthExceeded { path } => {
                write!(
                    f,
                    "Include depth limit exceeded (max 10) while including '{}'",
                    path
                )
            }
            ConfigParseError::UnexpectedCharacter {
                character,
                position,
                line,
            } => {
                write!(
                    f,
                    "Unexpected character '{}' at line {}, position {}",
                    character, line, position
                )
            }
            ConfigParseError::TokenNotAvailable => write!(f, "Token not available"),
        }
    }
}

impl std::error::Error for KanskeError {}
impl std::error::Error for ConfigParseError {}

pub type AppResult<T> = std::result::Result<T, KanskeError>;
pub type ParseResult<T> = std::result::Result<T, ConfigParseError>;
