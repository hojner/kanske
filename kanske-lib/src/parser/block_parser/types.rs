use std::mem::discriminant;
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
    Output(OutputConfig),
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: Option<String>,
    pub outputs: Vec<OutputConfig>,
    pub execs: Vec<ExecDirective>,
}

impl Profile {
    fn new(s: String) -> Self {
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
        let mut config = Config { items: Vec::new() };
        let mut config_item;

        while !self.is_at_end() {
            config_item = match &self.tokens[self.current] {
                Token::Profile => self.parse_profile()?,
                // Token::Include => ConfigItem::Include(self.parse_include()),
                Token::Output => ConfigItem::Output(self.parse_output()?),
                Token::Eof => break,
                other => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                        "Unexpected 352 token: {:?}",
                        other
                    )));
                }
            };
            config.items.push(config_item);
            self.advance();
        }

        dbg!(&config);

        Ok(config)
    }

    fn parse_profile(&mut self) -> AppResult<ConfigItem> {
        assert!(self.check(&Token::Profile));
        self.advance();

        let mut profile;
        if let Token::String(s) = &self.tokens[self.current] {
            // For now profile must have name,
            // will handle name generation for anonymous profiles later
            profile = Profile::new(s.clone());
            self.advance();
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Profile has no name".to_string(),
            ));
        }

        assert!(self.check(&Token::LeftBrace));
        self.advance();

        while !self.is_at_end() {
            match &self.tokens[self.current] {
                Token::Output => {
                    profile.outputs.push(self.parse_output()?);
                    println!("Output result: {:?}", profile);
                }
                Token::Exec => todo!(),
                Token::RightBrace => {
                    self.advance();
                    break;
                }
                other => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                        "Unexpected token: {:?}",
                        other
                    )));
                }
            };
        }
        Ok(ConfigItem::Profile(profile))
    }

    fn parse_output(&mut self) -> AppResult<OutputConfig> {
        assert!(self.check(&Token::Output));
        self.advance();
        let desc = if let Token::Identifier(desc) = &self.tokens[self.current] {
            OutputDesc::Name(desc.clone())
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Unexpected output name format".to_string(),
            ));
        };
        self.advance();

        let mut commands = Vec::new();
        loop {
            match &self.tokens[self.current] {
                Token::LeftBrace => {
                    dbg!(&self.tokens[self.current]);
                    self.advance();
                    dbg!(&self.tokens[self.current]);
                    while !self.is_at_end() && !self.check(&Token::RightBrace) {
                        commands.push(self.parse_output_command()?);
                        self.advance();
                    }
                    break;
                }
                Token::Eof | Token::Exec | Token::Include | Token::Output | Token::Profile => {
                    break;
                }
                Token::RightBrace => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "Found right brace before left brace".to_string(),
                    ));
                }
                Token::Enable
                | Token::Disable
                | Token::Mode
                | Token::Position
                | Token::Scale
                | Token::Transform
                | Token::AdaptiveSync => {
                    commands.push(self.parse_output_command()?);
                    self.advance();
                }
                _ => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                        "Unexpected token found: {:?}",
                        &self.tokens[self.current]
                    )));
                }
            };
        }

        Ok(OutputConfig { desc, commands })
    }

    fn parse_output_command(&mut self) -> AppResult<OutputCommand> {
        match &self.tokens[self.current] {
            Token::Enable | Token::Disable => self.parse_able(),
            Token::Mode => self.parse_mode(),
            Token::Position => self.parse_position(),
            Token::Scale => self.parse_scale(),
            Token::Transform => self.parse_transform(),
            Token::AdaptiveSync => self.parse_adaptive_sync(),
            other => {
                return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                    "Unexpected output config format, found token: {:?}",
                    other
                )));
            }
        }
    }

    fn parse_able(&self) -> AppResult<OutputCommand> {
        assert!(self.check(&Token::Enable) || self.check(&Token::Disable));
        if self.check(&Token::Enable) {
            return Ok(OutputCommand::Enable);
        } else if self.check(&Token::Disable) {
            return Ok(OutputCommand::Disable);
        }
        Err(KanskeError::ParsedStringUnexpectedFormat(
            "Cannot parse Enable/Disable".to_string(),
        ))
    }

    fn parse_mode(&mut self) -> AppResult<OutputCommand> {
        assert!(self.check(&Token::Mode));
        self.advance();
        dbg!(&self.tokens[self.current]);

        let (width, height, frequency) =
            if let Token::Identifier(mode_str) = &self.tokens[self.current] {
                self.parse_mode_str(mode_str)?
            } else {
                return Err(KanskeError::ParsedStringUnexpectedFormat(
                    "Unexpected format".to_string(),
                ));
            };

        Ok(OutputCommand::Mode {
            width,
            height,
            frequency,
        })
    }

    fn parse_mode_str(&self, s: &str) -> AppResult<(u32, u32, Option<f32>)> {
        let parts: Vec<_> = s.split("@").collect();
        let resolution = parts[0];
        let res_parts: Vec<_> = resolution.split("x").collect();

        if res_parts.len() != 2 {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Wrong resolution format, use <width in pixels>X<height in pixels>".to_string(),
            ));
        }

        let frequency = if parts.len() > 1 {
            let freq_str = parts[1].trim().trim_end_matches("Hz");
            Some(freq_str.parse::<f32>().map_err(|_| {
                KanskeError::ParsedStringUnexpectedFormat("Invalid frequency format".to_string())
            })?)
        } else {
            None
        };

        let width = res_parts[0].trim().parse::<u32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat("Wrong resolution width format".to_string())
        })?;
        let height = res_parts[1].trim().parse::<u32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat("Wrong resolution height format".to_string())
        })?;

        Ok((width, height, frequency))
    }

    fn parse_position(&mut self) -> AppResult<OutputCommand> {
        assert!(self.check(&Token::Position));
        self.advance();

        let (x, y) = if let Token::Identifier(position_str) = &self.tokens[self.current] {
            self.parse_position_str(position_str)?
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Unexpected format".to_string(),
            ));
        };

        Ok(OutputCommand::Position { x, y })
    }

    fn parse_position_str(&self, s: &str) -> AppResult<(i32, i32)> {
        let parts: Vec<_> = s.split(",").collect();

        if parts.len() != 2 {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Position parts must be separated by a comma".to_string(),
            ));
        }

        let x = parts[0].trim().parse::<i32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Cannot parse X value in position".to_string(),
            )
        })?;

        let y = parts[1].trim().parse::<i32>().map_err(|_| {
            KanskeError::ParsedStringUnexpectedFormat(
                "Cannot parse Y value in position".to_string(),
            )
        })?;

        Ok((x, y))
    }

    fn parse_scale(&mut self) -> AppResult<OutputCommand> {
        assert!(self.check(&Token::Scale));
        self.advance();

        let s = if let Token::Number(scale_str) = &self.tokens[self.current] {
            *scale_str
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Unexpected format".to_string(),
            ));
        };

        Ok(OutputCommand::Scale(s))
    }

    fn parse_transform(&mut self) -> AppResult<OutputCommand> {
        assert!(self.check(&Token::Transform));
        self.advance();

        let transform = match &self.tokens[self.current] {
            Token::Number(n) => match *n as i32 {
                90 => Transform::Rotate90,
                180 => Transform::Rotate180,
                270 => Transform::Rotate270,
                _ => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                        "Invalid transform type: {}",
                        n
                    )));
                }
            },
            Token::Identifier(s) => match s.as_str() {
                "normal" => Transform::Normal,
                "flipped" => Transform::Flipped,
                "flipped-90" => Transform::Flipped90,
                "flipped-180" => Transform::Flipped180,
                "flipped-270" => Transform::Flipped270,
                _ => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                        "Invalid transform type: {}",
                        s
                    )));
                }
            },
            other => {
                return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                    "Unexpected token: {:?}",
                    other
                )));
            }
        };

        Ok(OutputCommand::Transform(transform))
    }

    fn parse_adaptive_sync(&mut self) -> AppResult<OutputCommand> {
        assert!(self.check(&Token::AdaptiveSync));
        self.advance();

        let adaptive_sync = if let Token::Identifier(a) = &self.tokens[self.current] {
            match a.as_str() {
                "on" => OutputCommand::AdaptiveSync(true),
                "off" => OutputCommand::AdaptiveSync(false),
                _ => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(format!(
                        "Cannot parse the adaptive sync setting: {}",
                        a
                    )));
                }
            }
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Cannot parse the adaptive sync setting".to_string(),
            ));
        };

        Ok(adaptive_sync)
    }

    fn check(&self, token: &Token) -> bool {
        discriminant(&self.tokens[self.current]) == discriminant(token)
    }

    // fn peek(&self) -> Token {
    //     self.tokens[self.current + 1].clone()
    // }

    fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true;
        }
        false
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
