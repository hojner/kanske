use std::{collections::BTreeMap, sync::Arc};

use crate::{AppResult, KanskeError};

// --------------------------
// This code block is for the Params type
// -------------------------

#[derive(Debug)]
pub struct Params {
    pub name: Option<String>,
    pub enable: Option<bool>,
    pub mode: Option<Mode>,
    pub position: Option<Position>,
    pub scale: Option<Scale>,
    pub transform: Option<Transform>,
    pub adaptive_sync: Option<AdaptiveSync>,
    pub alias: Option<Alias>,
}

impl Params {
    pub fn new() -> Self {
        Params {
            name: None,
            enable: None,
            mode: None,
            position: None,
            scale: None,
            transform: None,
            adaptive_sync: None,
            alias: None,
        }
    }

    fn from_line(s: &str) -> AppResult<Params> {
        let line = s.trim();
        let mut param = Params::new();
        let mut line_parts = line.split_whitespace();
        while let Some(txt) = line_parts.next() {
            if txt == "output" {
                param.name = line_parts.next().map(|s| s.to_string());
            } else if txt == "enable" {
                param.enable = Some(true);
            } else if txt == "disable" {
                param.enable = Some(false);
            } else if txt == "mode" {
                param.mode = line_parts.next().map(|s| Mode::from_line(s)).transpose()?;
            } else if txt == "position" {
                param.position = line_parts
                    .next()
                    .map(|s| Position::from_line(s))
                    .transpose()?;
            } else if txt == "scale" {
                param.scale = line_parts.next().map(|s| Scale::from_line(s)).transpose()?;
            } else if txt == "transform" {
                param.transform = line_parts
                    .next()
                    .map(|s| Transform::from_line(s))
                    .transpose()?;
            } else if txt == "adaptive_sync" {
                param.adaptive_sync = line_parts
                    .next()
                    .map(|s| AdaptiveSync::from_line(s))
                    .transpose()?;
            } else if txt == "alias" {
                param.alias = line_parts.next().map(|s| Alias::from_line(s)).transpose()?;
            }
        }
        Ok(param)
    }

    fn count_some(&self) -> usize {
        [
            self.name.is_some(),
            self.enable.is_some(),
            self.mode.is_some(),
            self.position.is_some(),
            self.scale.is_some(),
            self.transform.is_some(),
            self.adaptive_sync.is_some(),
            self.alias.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

// --------------------------
// This code block is for sub-types of the Params type
// -------------------------
//
// -------------------------
// Mode type and impl
// -------------------------

#[derive(Debug)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub frequency: Option<f32>,
}

impl Mode {
    pub fn from_line(line: &str) -> AppResult<Mode> {
        let mode_str = line.trim();

        let (dimensions, freq_part) = if let Some((dims, freq)) = mode_str.split_once('@') {
            (dims, Some(freq))
        } else {
            (mode_str, None)
        };

        let (width_str, height_str) = dimensions.split_once('x').ok_or_else(|| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Missing 'x' separator in dimensions".to_string(),
            )
        })?;

        let width = width_str.parse::<u32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat("Width cannot be parsed".to_string())
        })?;

        let height = height_str.parse::<u32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat("Height cannot be parsed".to_string())
        })?;

        let frequency = if let Some(freq_str) = freq_part {
            let freq_num = freq_str.strip_suffix("Hz").ok_or_else(|| {
                KanskeError::ParsedStringUnexpectedFormat(
                    "Frequency must end with 'Hz'".to_string(),
                )
            })?;

            Some(freq_num.parse::<f32>().map_err(|_| {
                KanskeError::ParsedStringUnexpectedFormat("Frequency cannot be parsed".to_string())
            })?)
        } else {
            None
        };

        Ok(Self {
            width,
            height,
            frequency,
        })
    }
}

// -------------------
// Position and impl
// ------------------

#[derive(Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn from_line(line: &str) -> AppResult<Self> {
        let mut position_str = line.split(",");

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

// --------------------
// Scale and impl
// --------------------

#[derive(Debug)]
pub struct Scale(pub f32);

impl Scale {
    fn from_line(s: &str) -> AppResult<Self> {
        let parsed = s.parse::<f32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Error in Scale field float parsing".to_string(),
            )
        })?;
        Ok(Self(parsed))
    }
}

// ------------------------
// Transform and impl
// ------------------------

#[derive(Debug)]
pub enum Transform {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl Transform {
    fn from_line(s: &str) -> AppResult<Self> {
        match s.trim() {
            "normal" => Ok(Transform::Normal),
            "90" => Ok(Transform::Rotate90),
            "180" => Ok(Transform::Rotate180),
            "270" => Ok(Transform::Rotate270),
            "flipped" => Ok(Transform::Flipped),
            "flipped-90" => Ok(Transform::Flipped90),
            "flipped-180" => Ok(Transform::Flipped180),
            "flipped-270" => Ok(Transform::Flipped270),
            _ => Err(KanskeError::ParsedStringUnexpectedFormat(
                "Transform field parsing error".to_string(),
            )),
        }
    }
}

// ---------------------
// Adaptive Sync and impl
// ---------------------

#[derive(Debug)]
pub struct AdaptiveSync(pub bool);

impl AdaptiveSync {
    fn from_line(s: &str) -> AppResult<Self> {
        match s.trim() {
            "on" => Ok(Self(true)),
            "off" => Ok(Self(false)),
            _ => Err(KanskeError::ParsedStringUnexpectedFormat(
                "Adaptive Sync parse error".to_string(),
            )),
        }
    }
}

// ---------------------
// Alias and impl
// ---------------------

#[derive(Debug)]
pub struct Alias(pub Arc<str>);

impl Alias {
    fn from_line(s: &str) -> AppResult<Self> {
        let s_trim = s.trim();
        if !s_trim.starts_with("$") || s_trim.is_empty() || s_trim == "$" {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Wrong format alias string".to_string(),
            ));
        } else {
            let alias = Arc::from(
                s_trim
                    .strip_prefix("$")
                    .expect("We have checked the string already"),
            );
            Ok(Self(alias))
        }
    }
}

// --------------------------
// Directive type and impl
// --------------------------

#[derive(Debug)]
pub struct Directive {
    pub name: Arc<str>,
    pub params: Params,
    pub params_len: usize,
    pub children: Option<Box<Vec<Directive>>>,
    pub line_no: usize,
}

impl Directive {
    pub fn from_line(map: BTreeMap<usize, &str>) -> AppResult<Self> {
        let (line_no, params_str) = map.first_key_value().ok_or_else(|| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Could not parse map for the first line into Directive".to_string(),
            )
        })?;
        let name = if params_str.starts_with("enable") || params_str.starts_with("disable") {
            "enable"
        } else {
            params_str.split_whitespace().next().ok_or_else(|| {
            KanskeError::ParsedStringUnexpectedFormat(format!(
                "Directive has the wrong format, should be <name> <parameters>. Config line: {}",
                line_no
            ))
            })?
        };
        let params = Params::from_line(params_str.trim())?;
        let params_len = params.count_some();

        Ok(Self {
            name: Arc::from(name),
            params,
            params_len,
            children: None,
            line_no: *line_no,
        })
    }
}
