// Token types for lexical analysis

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Token {
    Profile,
    Output,
    Exec,
    Include,
    Enable,
    Disable,
    Mode,
    Position,
    Scale,
    Transform,
    AdaptiveSync,

    LeftBrace,
    RightBrace,

    String(String),
    Identifier(String),
    Number(f32),

    Eof,
}
