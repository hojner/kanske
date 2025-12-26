use std::{collections::BTreeMap, sync::Arc};

use crate::{AppResult, KanskeError};

// -------------------------------
// Start of alt types
// -------------------------------

#[derive(Debug, Clone)]
pub struct Config {
    pub items: Vec<ConfigItem>,
}

#[derive(Debug, Clone)]
pub enum ConfigItem {
    Profile(Profile),
    Include(IncludeDirective),
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: Option<String>,
    pub outputs: Vec<OutputConfig>,
    pub execs: Vec<ExecDirective>,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub criteria: OutputCriteria,
    pub commands: Vec<OutputCommand>,
}

#[derive(Debug, Clone)]
pub enum OutputCriteria {
    Name(String),
    Description(String),
    Any, // "*"
}

#[derive(Debug, Clone)]
pub enum OutputCommand {
    Enable,
    Disable,
    Mode {
        width: u32,
        height: u32,
        refresh: Option<f32>,
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

// -------------------------------
// End of alt types
// -------------------------------

// Tokeizer
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

    pub fn tokenizer(&mut self) -> AppResult<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                tokens.push(Token::Eof);
                break;
            }
            let token = self.new_token()?;
            dbg!(&token);
            tokens.push(token);
        }
        dbg!(&tokens);
        Ok(tokens)
    }

    fn new_token(&mut self) -> AppResult<Token> {
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
            _ => Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                "Unexpected char: {}",
                ch,
            ))),
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

    fn read_number(&mut self) -> AppResult<Token> {
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
        let number = text.parse::<f32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat("Cannot parse number to f32".to_string())
        })?;
        Ok(Token::Number(number))
    }

    fn read_quoted_string(&mut self) -> AppResult<Token> {
        self.advance();
        let start = self.position;

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Non-terminated quote".to_string(),
            ));
        }
        let name_string = self.input[start..self.position].to_string();
        self.advance();

        Ok(Token::String(name_string))
    }

    fn read_identifier(&mut self) -> AppResult<Token> {
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

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> AppResult<Config> {
        // Assert that the first item is profile or output, or EOF if the config was empty
        // Also assert that the last item is always EOF.
        assert!(
            self.tokens.first().expect("first item must exist") == &Token::Output
                || self.tokens.first().expect("first item must exist") == &Token::Profile
                || self.tokens.first().expect("first item must exist") == &Token::Eof
        );
        assert!(self.tokens.last().expect("last item must exist") == &Token::Eof);
        // --------------
        // --------------

        while !self.is_at_end() {
            match self.tokens[self.current] {
                Token::Profile => self.parse_profile()?,
                Token::Include => self.parse_include()?,
                Token::Output => self.parse_output()?,
                _ => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "Unexpected token".to_string(),
                    ));
                }
            };

            self.advance();
        }

        todo!()
    }

    fn parse_profile(&mut self) -> AppResult<ConfigItem> {
        // ----------------
        // For now profile must have name,
        // will handle name generation for anonymous profiles later
        // ----------------
        self.advance();

        if let Token::String(s) = &self.tokens[self.current] {
            let profile = Profile {
                name: Some(s.to_owned()),
                outputs: Vec::new(),
                execs: Vec::new(),
            };
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Profile has no name".to_string(),
            ));
        }

        self.advance();
        assert!(&self.tokens[self.current] == &Token::LeftBrace);

        todo!();
    }

    fn parse_include(&mut self) -> AppResult<ConfigItem> {
        todo!();
    }

    fn parse_output(&mut self) -> AppResult<ConfigItem> {
        todo!();
    }

    fn parse_braces(&mut self) -> AppResult<usize> {
        let mut left_brace_count = 1;
        let mut right_brace_count = 0;

        let start = self.current;
        let token = &self.tokens[start];
        assert!(token == &Token::LeftBrace);

        while left_brace_count > right_brace_count {
            if self.is_at_end() {
                return Err(KanskeError::ParsedStringUnexpectedFormat(
                    "Braces not matching".to_string(),
                ));
            }
            self.advance();
            match &self.tokens[self.current] {
                Token::LeftBrace => left_brace_count += 1,
                Token::RightBrace => right_brace_count += 1,
                _ => continue,
            }
        }
        todo!()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current + 1].clone()
    }

    fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true;
        }
        return false;
    }

    fn advance(&mut self) {
        self.current += 1;
    }
}

//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//k
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
            // } else if txt == "transform" {
            //     param.transform = line_parts
            //         .next()
            //         .map(|s| Transform::from_line(s))
            //         .transpose()?;
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
