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
pub struct Transform(pub Arc<str>);

#[derive(Debug)]
pub struct AdaptiveSync(pub bool);

#[derive(Debug)]
pub struct Alias(pub Arc<str>);

#[derive(Debug)]
pub struct Params {
    pub name: Arc<str>,
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
            name: Arc::from(""),
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
