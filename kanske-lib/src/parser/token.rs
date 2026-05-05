// Token types for lexical analysis

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Token {
    Profile,
    Output,
    Exec,
    Include,
    Enabled(bool),
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

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TokenPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TokenHolder {
    pub token: Token,
    pub position: TokenPosition,
}

impl std::fmt::Display for TokenPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {} and col {}", self.line + 1, self.column)
    }
}
