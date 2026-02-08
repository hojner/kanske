// AST (Abstract Syntax Tree) types for Kanske configuration

#[derive(Debug, Clone)]
pub struct Config {
    pub items: Vec<ConfigItem>,
}

#[derive(Debug, Clone)]
pub enum ConfigItem {
    Profile(Profile),
    Include(IncludeDirective),
    Output(OutputConfig),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum OutputDesc {
    Name(String),
    Any,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub desc: OutputDesc,
    pub commands: Vec<OutputCommand>,
}

#[derive(Debug, Clone)]
pub enum OutputCommand {
    Enable,
    Disable,
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ExecDirective {
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct IncludeDirective {
    pub path: String,
}
