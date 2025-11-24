use super::types::*;
use crate::{AppResult, KanskeError};
use std::str::FromStr;
use std::sync::Arc;

// -------------------------
// FromStr for Mode
// -------------------------

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

// -------------------------
// FromStr for Position
// -------------------------

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

// -------------------------
// FromStr for Scale
// -------------------------

impl FromStr for Scale {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        s.parse::<f32>()
            .map(Scale)
            .map_err(|_| KanskeError::ParsedStringUnexpectedFormat("scale".to_string()))
    }
}

// -------------------------
// FromStr for Transform
// -------------------------

impl FromStr for Transform {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        Ok(Transform(Arc::from(s)))
    }
}

// -------------------------
// FromStr for AdaptiveSync
// -------------------------

impl FromStr for AdaptiveSync {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        match s {
            "on" => Ok(AdaptiveSync(true)),
            "off" => Ok(AdaptiveSync(false)),
            _ => Err(KanskeError::ParsedStringUnexpectedFormat(
                "adaptive sync setting format".to_string(),
            )),
        }
    }
}

// ------------------------
// FromStr for Alias
// ------------------------

impl FromStr for Alias {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        Ok(Alias(Arc::from(s)))
    }
}

// ------------------------
// FromStr for Params
// ------------------------

impl FromStr for Params {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        let mut parts = s.split_whitespace();
        let mut params = Params::new();

        dbg!(&parts);

        params.enable = parts.next().and_then(|e| {
            if e == "enable" {
                println!("This is in the enable block");
                Some(true)
            } else if e == "disable" {
                Some(false)
            } else {
                None
            }
        });
        while let Some(keyword) = parts.next() {
            match keyword {
                "mode" => {
                    let mode_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no mode part".to_string())
                    })?;
                    params.mode = Some(Mode::from_str(mode_str)?);
                }
                "position" => {
                    let pos_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no position part".to_string())
                    })?;
                    params.position = Some(Position::from_str(pos_str)?);
                }
                "scale" => {
                    let scale_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no scale part".to_string())
                    })?;
                    params.scale = Some(Scale::from_str(scale_str)?);
                }
                "transform" => {
                    let transform_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no transform part".to_string())
                    })?;
                    params.transform = Some(Transform::from_str(transform_str)?);
                }
                "adaptive_sync" => {
                    let adaptive_sync_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat(
                            "no adaptive_sync part".to_string(),
                        )
                    })?;
                    params.adaptive_sync = Some(AdaptiveSync::from_str(adaptive_sync_str)?);
                }
                "alias" => {
                    let alias_str = parts.next().ok_or_else(|| {
                        KanskeError::ParsedStringUnexpectedFormat("no alias part".to_string())
                    })?;
                    params.alias = Some(Alias::from_str(alias_str)?);
                }
                other => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        format!("unexpected element in output string: {}", other).to_string(),
                    ));
                }
            }
        }
        Ok(params)
    }
}
