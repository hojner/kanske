use std::{collections::BTreeMap, sync::Arc};

use crate::{AppResult, KanskeError};

// --------------------------
// This code block is for the Params type
// -------------------------

#[derive(Debug)]
pub struct Params {
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
            enable: None,
            mode: None,
            position: None,
            scale: None,
            transform: None,
            adaptive_sync: None,
            alias: None,
        }
    }
}

impl Params {
    fn from_line(line: &str) -> AppResult<Params> {
        dbg!(&line);
        let (first, second) = line.trim().split_once(" ").ok_or_else(|| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Cannot split Directive into two at whitespace".to_string(),
            )
        })?;
        let mode;
        match first.trim() {
            "mode" => mode = Some(Mode::from_line(second)?),
            "position" => todo!("position"),
            "scale" => todo!("scale"),
            "transform" => todo!("transform"),
            "adaptive_sync" => todo!("adaptive_sync"),
            "alias" => todo!("alias"),
            _ => todo!("What to do with the rest?"),
        };

        Ok(Params {
            enable: None,
            mode,
            position: None,
            scale: None,
            transform: None,
            adaptive_sync: None,
            alias: None,
        })
    }
}

// --------------------------
// This code block is for sub-types of the Params type
// -------------------------

#[derive(Debug)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub frequency: Option<f32>,
}

impl Mode {
    pub fn from_line(line: &str) -> AppResult<Mode> {
        // let (_, mode_str) = line.split_once(' ').ok_or_else(|| {
        //     KanskeError::ParsedStringUnexpectedFormat("Wrong mode string format".to_string())
        // })?;

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

#[derive(Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct Scale(pub f32);

#[derive(Debug)]
pub struct Transform(pub Arc<str>);

#[derive(Debug)]
pub struct AdaptiveSync(pub bool);

#[derive(Debug)]
pub struct Alias(pub Arc<str>);

// --------------------------
// This code block is for the Directive type
// -------------------------

#[derive(Debug)]
pub struct Directive {
    pub name: Arc<str>,
    pub params: Params,
    pub children: Option<Box<Directive>>,
    pub line_no: usize,
}

impl Directive {
    pub fn from_line(map: BTreeMap<usize, &str>) -> AppResult<Self> {
        let (line, params_str) = map.first_key_value().ok_or_else(|| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Could not parse map for the first line into Directive".to_string(),
            )
        })?;
        let (name, params) = params_str.split_once(" ").ok_or_else(|| {
            KanskeError::ParsedStringUnexpectedFormat(format!(
                "Directive has the wrong format, should be <name> <parameters>. Config line: {}",
                line
            ))
        })?;
        if name == "profile" {}
        let params = Params::from_line(params_str)?;
        dbg!(name, &params);
        Ok(Self {
            name: Arc::from(name),
            params,
            children: None,
            line_no: 1,
        })
    }
}
