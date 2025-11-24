use std::{str::FromStr, sync::Arc};

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

// --------------------------
// This code block is for sub-types of the Params type
// -------------------------

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
pub struct Transform(pub Arc<str>);

#[derive(Debug)]
pub struct AdaptiveSync(pub bool);

#[derive(Debug)]
pub struct Alias(pub Arc<str>);

// --------------------------
// This code block is for the Block type
// -------------------------

// #[derive(Debug)]
// pub struct Block {
//     pub directives: Arc<[Directive]>,
//     pub directives_len: usize,
// }

// impl Block {
//     pub fn from_line(line_no: usize, line: &str) -> AppResult<Self> {
//         Ok(Self {
//             directives: Arc::new([Directive::from_line(line_no, line, 1)?]),
//             directives_len: 1,
//         })
//     }
// }

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
    pub fn from_line(line_no: usize, line: &str) -> AppResult<Self> {
        let (name, params) = if let Some((name, param_str)) = line.split_once(" ") {
            if param_str.ends_with("{") {
                println!("Ends with {{");
            }
            let params = Params::from_str(param_str)?;
            let name = name;
            (name, params)
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "directive str line".to_string(),
            ));
        };
        Ok(Self {
            name: Arc::from(name),
            params,
            children: None,
            line_no,
        })
    }
}
