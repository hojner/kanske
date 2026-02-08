// Lexical analyzer (tokenizer) for Kanske configuration files

use crate::error::{ParseResult, ConfigParseError};
use crate::parser::token::Token;

pub struct Lexer {
    pub input: String,
    pub position: usize,
    pub line: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 0,
        }
    }

    pub fn tokenizer(&mut self) -> ParseResult<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                tokens.push(Token::Eof);
                break;
            }
            let token = self.new_token()?;
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn new_token(&mut self) -> ParseResult<Token> {
        let ch = self.peek();

        match ch {
            '{' => {
                self.advance();
                return Ok(Token::LeftBrace);
            }
            '}' => {
                self.advance();
                return Ok(Token::RightBrace);
            }
            '"' => self.read_quoted_string(),
            _ if ch.is_alphabetic() || ch == '*' => self.read_identifier(),
            _ if ch.is_numeric() || ch == '-' => {
                if self.is_mode_str() {
                    self.read_identifier()
                } else if self.is_position_str() {
                    self.read_identifier()
                } else {
                    self.read_number()
                }
            }
            _ => Err(ConfigParseError::UnexpectedCharacter {
                character: ch,
                position: self.position,
                line: self.line,
            }),
        }
    }

    fn is_position_str(&self) -> bool {
        for ch in self.input[self.position..].chars() {
            if ch == ',' {
                return true;
            } else if ch.is_whitespace() || ch == '{' || ch == '}' {
                return false;
            }
        }
        false
    }

    fn is_mode_str(&self) -> bool {
        for ch in self.input[self.position..].chars() {
            if ch == 'x' || ch == '@' {
                return true;
            } else if ch.is_whitespace() || ch == '{' || ch == '}' {
                return false;
            }
        }
        false
    }

    fn read_number(&mut self) -> ParseResult<Token> {
        let start = self.position;

        if self.peek() == '-' {
            self.advance();
        }

        while !self.is_at_end() && self.peek().is_numeric() {
            self.advance();
        }
        if !self.is_at_end() && self.peek() == '.' {
            self.advance();
        }
        while !self.is_at_end() && self.peek().is_numeric() {
            self.advance();
        }

        let text = &self.input[start..self.position];
        let number = text
            .parse::<f32>()
            .map_err(|_| ConfigParseError::InvalidNumber {
                value: text.to_string(),
                position: start,
            })?;
        Ok(Token::Number(number))
    }

    fn read_quoted_string(&mut self) -> ParseResult<Token> {
        let start_line = self.line;
        self.advance();
        let start = self.position;

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(ConfigParseError::UnterminatedString { line: start_line });
        }
        let name_string = self.input[start..self.position].to_string();
        self.advance();

        Ok(Token::String(name_string))
    }

    fn read_identifier(&mut self) -> ParseResult<Token> {
        let start = self.position;

        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric()
                || ch == '_'
                || ch == '-'
                || ch == '*'
                || ch == '.'
                || ch == ','
                || ch == '@'
            {
                self.advance();
            } else {
                break;
            }
        }
        let name_string = &self.input[start..self.position];
        let token = match name_string {
            "profile" => Token::Profile,
            "output" => Token::Output,
            "exec" => Token::Exec,
            "include" => Token::Include,
            "enable" => Token::Enable,
            "disable" => Token::Disable,
            "mode" => Token::Mode,
            "position" => Token::Position,
            "scale" => Token::Scale,
            "transform" => Token::Transform,
            "adaptive_sync" => Token::AdaptiveSync,
            "{" => Token::LeftBrace,
            "}" => Token::RightBrace,
            _ => Token::Identifier(name_string.to_string()),
        };

        Ok(token)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '#' => {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.input[self.position..].chars().next() {
            self.position += ch.len_utf8();
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn peek(&self) -> char {
        self.input[self.position..].chars().next().unwrap_or('\0')
    }
}
