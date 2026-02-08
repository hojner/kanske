#[derive(Debug)]
pub enum KanskeError {
    ConfigError {
        file: String,
        source: ConfigParseError,
    },
    WaylandConnectError(wayland_client::ConnectError),
    ReadIOError(std::io::Error),
    WaylandDispatchError(wayland_client::DispatchError),
}

#[derive(Debug)]
pub enum ConfigParseError {
    ParsedStringIsEmpty,
    ParsedStringUnexpectedFormat(String),
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
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
    },
    InvalidPosition {
        value: String,
        reason: String,
    },
    InvalidTransform {
        value: String,
    },
    InvalidAdaptiveSync {
        value: String,
    },
    MissingProfileName {
        position: usize,
    },
    UnexpectedCharacter {
        character: char,
        position: usize,
        line: usize,
    },
}

impl From<wayland_client::DispatchError> for KanskeError {
    fn from(err: wayland_client::DispatchError) -> Self {
        KanskeError::WaylandDispatchError(err)
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
            KanskeError::WaylandConnectError(e) => write!(f, "Wayland connection error: {}", e),
            KanskeError::ReadIOError(e) => write!(f, "I/O error: {}", e),
            KanskeError::WaylandDispatchError(e) => write!(f, "Dispatch error: {}", e),
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
                    "Unexpected token at position {}: expected {}, found {}",
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
            ConfigParseError::InvalidResolution { value, reason } => {
                write!(f, "Invalid resolution '{}': {}", value, reason)
            }
            ConfigParseError::InvalidPosition { value, reason } => {
                write!(f, "Invalid position '{}': {}", value, reason)
            }
            ConfigParseError::InvalidTransform { value } => {
                write!(
                    f,
                    "Invalid transform '{}': must be one of: normal, 90, 180, 270, flipped, flipped-90, flipped-180, flipped-270",
                    value
                )
            }
            ConfigParseError::InvalidAdaptiveSync { value } => {
                write!(
                    f,
                    "Invalid adaptive_sync value '{}': must be 'on' or 'off'",
                    value
                )
            }
            ConfigParseError::MissingProfileName { position } => {
                write!(f, "Missing profile name at position {}", position)
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
        }
    }
}

impl std::error::Error for KanskeError {}
impl std::error::Error for ConfigParseError {}

pub type AppResult<T> = std::result::Result<T, KanskeError>;
pub type ParseResult<T> = std::result::Result<T, ConfigParseError>;
