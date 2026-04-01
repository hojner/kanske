// AST (Abstract Syntax Tree) types for Kanske configuration

use wayland_client::protocol::wl_output;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub items: Vec<ConfigItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigItem {
    Profile(Profile),
    Include(IncludeDirective),
    Output(OutputConfig),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub name: Option<String>,
    pub outputs: Vec<OutputConfig>,
    pub execs: Vec<ExecDirective>,
}

impl Profile {
    pub(crate) fn new(s: String) -> Self {
        Self {
            name: Some(s),
            outputs: Vec::new(),
            execs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputDesc {
    Name(String),
    Any,
}

impl OutputDesc {
    pub fn matches(&self, head_name: &str) -> bool {
        match self {
            OutputDesc::Name(n) => n == head_name,
            OutputDesc::Any => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputConfig {
    pub desc: OutputDesc,
    pub commands: Vec<OutputCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputCommand {
    Enabled(bool),
    Mode {
        width: u32,
        height: u32,
        frequency: Option<f32>,
    },
    Position {
        x: i32,
        y: i32,
    },
    Scale(f32),
    Transform(Transform),
    AdaptiveSync(bool),
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl From<Transform> for wl_output::Transform {
    fn from(t: Transform) -> Self {
        match t {
            Transform::Normal => wl_output::Transform::Normal,
            Transform::Rotate90 => wl_output::Transform::_90,
            Transform::Rotate180 => wl_output::Transform::_180,
            Transform::Rotate270 => wl_output::Transform::_270,
            Transform::Flipped => wl_output::Transform::Flipped,
            Transform::Flipped90 => wl_output::Transform::Flipped90,
            Transform::Flipped180 => wl_output::Transform::Flipped180,
            Transform::Flipped270 => wl_output::Transform::Flipped270,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecDirective {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeDirective {
    pub path: String,
}
