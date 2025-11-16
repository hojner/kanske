use crate::{AppResult, KanskeError};
pub use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub frequency: f32,
}

#[derive(Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct Scale(pub f32);

#[derive(Debug)]
pub struct Output {
    pub name: Arc<str>,
    pub enable: bool,
    pub mode: Option<Mode>,
    pub position: Option<Position>,
    pub scale: Option<Scale>,
}

impl FromStr for Mode {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        let mode_str = s.replace("x", " ").replace("@", " ").replace("Hz", " ");
        let mut mode_parts = mode_str.split_whitespace();
        let width = mode_parts
            .next()
            .and_then(|w| w.parse().ok())
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("width".to_string()))?;

        let height = mode_parts
            .next()
            .and_then(|h| h.parse().ok())
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("height".to_string()))?;

        let frequency = mode_parts
            .next()
            .and_then(|f| f.parse().ok())
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("frequency".to_string()))?;

        Ok(Mode {
            width,
            height,
            frequency,
        })
    }
}

impl FromStr for Position {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        let mut position_str = s.split(",");

        let x: u32 = position_str
            .next()
            .and_then(|x| x.parse::<u32>().ok())
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("position".to_string()))?;

        let y = position_str
            .next()
            .and_then(|y| y.parse().ok())
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("position".to_string()))?;

        Ok(Position { x, y })
    }
}

impl FromStr for Scale {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        s.parse::<f32>()
            .map(Scale)
            .map_err(|_| KanskeError::ParsedStringUnexpectedFormat("scale".to_string()))
    }
}

impl FromStr for Output {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        let mut parts = s.split_whitespace();
        if parts.next() != Some("output") {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "output keyword".to_string(),
            ));
        }
        let name: Arc<str> = parts
            .next()
            .map(Arc::from)
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("name".to_string()))?;

        let enable = parts
            .next()
            .and_then(|e| {
                if e == "enable" {
                    Some(true)
                } else if e == "disable" {
                    Some(false)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                KanskeError::ParsedStringUnexpectedFormat("enable string formatting".to_string())
            })?;
        let mut mode = None;
        let mut position = None;
        let mut scale = None;
        while let Some(keyword) = parts.next() {
            match keyword {
                "mode" => {
                    let mode_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no mode part".to_string())
                    })?;
                    mode = Some(Mode::from_str(mode_str)?);
                }
                "position" => {
                    let pos_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no position part".to_string())
                    })?;
                    position = Some(Position::from_str(pos_str)?);
                }
                "scale" => {
                    let scale_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no scale part".to_string())
                    })?;
                    scale = Some(Scale::from_str(scale_str)?);
                }
                _ => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "unexpected element in output string".to_string(),
                    ));
                }
            }
        }
        Ok(Output {
            name,
            enable,
            mode,
            position,
            scale,
        })
    }
}
