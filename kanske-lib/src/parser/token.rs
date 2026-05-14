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

#[derive(Debug, Clone)]
pub struct TokenStream {
    tokens: Vec<TokenHolder>,
    current: usize,
}

impl TokenStream {
    /// Builds a stream from a token list.
    /// Appends a synthetic Eof if the list is empty or doesn't already end with one.
    pub fn new(mut tokens: Vec<TokenHolder>) -> Self {
        match tokens.last() {
            Some(t) if t.token == Token::Eof => {}
            _ => tokens.push(TokenHolder {
                token: Token::Eof,
                position: tokens
                    .last()
                    .map_or(TokenPosition { line: 0, column: 0 }, |t| {
                        t.position.clone()
                    }),
            }),
        }
        Self { tokens, current: 0 }
    }

    /// Returns the token at the current cursor. Never panics.
    pub fn current(&self) -> &TokenHolder {
        &self.tokens[self.current]
    }

    /// Advances the cursor. Saturates at the Eof token — can never go past it.
    pub fn advance(&mut self) {
        if self.current + 1 < self.tokens.len() {
            self.current += 1;
        }
    }

    /// True when the current token is Eof.
    pub fn is_at_end(&self) -> bool {
        matches!(self.current().token, Token::Eof)
    }
}
