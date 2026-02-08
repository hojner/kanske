use std::mem::discriminant;

use crate::error::{ConfigParseError, ParseResult};
use crate::parser::ast::*;
use crate::parser::token::Token;

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Config> {
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
                    return Err(ConfigParseError::UnexpectedToken {
                        expected: "profile or output".to_string(),
                        found: format!("{:?}", other),
                        position: self.current,
                    });
                }
            };
            config.items.push(config_item);
        }
        Ok(config)
    }

    fn parse_profile(&mut self) -> ParseResult<ConfigItem> {
        assert!(self.check(&Token::Profile));
        self.advance();

        // For now profile must have name,
        // will handle name generation for anonymous profiles later

        let name = match &self.tokens[self.current] {
            Token::String(s) | Token::Identifier(s) => s.clone(),
            _ => {
                return Err(ConfigParseError::MissingProfileName {
                    position: self.current,
                });
            }
        };

        let mut profile = Profile::new(name);

        self.advance();
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
                    return Err(ConfigParseError::UnexpectedToken {
                        expected: "output, exec, or }".to_string(),
                        found: format!("{:?}", other),
                        position: self.current,
                    });
                }
            };
        }
        Ok(ConfigItem::Profile(profile))
    }

    fn parse_output(&mut self) -> ParseResult<OutputConfig> {
        assert!(self.check(&Token::Output));
        self.advance();

        let desc = if let Token::Identifier(desc) = &self.tokens[self.current] {
            OutputDesc::Name(desc.clone())
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "output name".to_string(),
                found: format!("{:?}", &self.tokens[self.current]),
                position: self.current,
            });
        };

        self.advance();

        let mut commands = Vec::new();
        loop {
            match &self.tokens[self.current] {
                Token::LeftBrace => {
                    self.advance();

                    while !self.is_at_end() && !self.check(&Token::RightBrace) {
                        commands.push(self.parse_output_command()?);
                        self.advance();
                    }
                    assert!(self.check(&Token::RightBrace));
                    self.advance();
                    break;
                }
                Token::RightBrace
                | Token::Eof
                | Token::Exec
                | Token::Include
                | Token::Output
                | Token::Profile => {
                    break;
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
                    return Err(ConfigParseError::UnexpectedToken {
                        expected: "output command (enable, disable, mode, position, scale, transform, adaptive_sync)".to_string(),
                        found: format!("{:?}", &self.tokens[self.current]),
                        position: self.current,
                    });
                }
            };
        }

        Ok(OutputConfig { desc, commands })
    }

    fn parse_output_command(&mut self) -> ParseResult<OutputCommand> {
        match &self.tokens[self.current] {
            Token::Enable | Token::Disable => self.parse_able(),
            Token::Mode => self.parse_mode(),
            Token::Position => self.parse_position(),
            Token::Scale => self.parse_scale(),
            Token::Transform => self.parse_transform(),
            Token::AdaptiveSync => self.parse_adaptive_sync(),
            other => {
                return Err(ConfigParseError::UnexpectedToken {
                    expected: "output command".to_string(),
                    found: format!("{:?}", other),
                    position: self.current,
                });
            }
        }
    }

    fn parse_able(&self) -> ParseResult<OutputCommand> {
        assert!(self.check(&Token::Enable) || self.check(&Token::Disable));
        if self.check(&Token::Enable) {
            return Ok(OutputCommand::Enable);
        } else if self.check(&Token::Disable) {
            return Ok(OutputCommand::Disable);
        }
        Err(ConfigParseError::ParsedStringUnexpectedFormat(
            "Cannot parse Enable/Disable".to_string(),
        ))
    }

    fn parse_mode(&mut self) -> ParseResult<OutputCommand> {
        assert!(self.check(&Token::Mode));
        self.advance();
        dbg!(&self.tokens[self.current]);

        let (width, height, frequency) =
            if let Token::Identifier(mode_str) = &self.tokens[self.current] {
                self.parse_mode_str(mode_str)?
            } else {
                return Err(ConfigParseError::UnexpectedToken {
                    expected: "mode string (e.g., 1920x1080@60Hz)".to_string(),
                    found: format!("{:?}", &self.tokens[self.current]),
                    position: self.current,
                });
            };

        Ok(OutputCommand::Mode {
            width,
            height,
            frequency,
        })
    }

    fn parse_mode_str(&self, s: &str) -> ParseResult<(u32, u32, Option<f32>)> {
        let parts: Vec<_> = s.split("@").collect();
        let resolution = parts[0];
        let res_parts: Vec<_> = resolution.split("x").collect();

        if res_parts.len() != 2 {
            return Err(ConfigParseError::InvalidResolution {
                value: s.to_string(),
                reason: "expected format: WIDTHxHEIGHT (e.g., 1920x1080)".to_string(),
            });
        }

        let frequency = if parts.len() > 1 {
            let freq_str = parts[1].trim().trim_end_matches("Hz");
            Some(
                freq_str
                    .parse::<f32>()
                    .map_err(|_| ConfigParseError::InvalidResolution {
                        value: s.to_string(),
                        reason: format!("invalid frequency: {}", parts[1]),
                    })?,
            )
        } else {
            None
        };

        let width = res_parts[0].trim().parse::<u32>().map_err(|_| {
            ConfigParseError::InvalidResolution {
                value: s.to_string(),
                reason: format!("invalid width: {}", res_parts[0]),
            }
        })?;
        let height = res_parts[1].trim().parse::<u32>().map_err(|_| {
            ConfigParseError::InvalidResolution {
                value: s.to_string(),
                reason: format!("invalid height: {}", res_parts[1]),
            }
        })?;

        Ok((width, height, frequency))
    }

    fn parse_position(&mut self) -> ParseResult<OutputCommand> {
        assert!(self.check(&Token::Position));
        self.advance();

        let (x, y) = if let Token::Identifier(position_str) = &self.tokens[self.current] {
            self.parse_position_str(position_str)?
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "position string (e.g., 1920,0)".to_string(),
                found: format!("{:?}", &self.tokens[self.current]),
                position: self.current,
            });
        };

        Ok(OutputCommand::Position { x, y })
    }

    fn parse_position_str(&self, s: &str) -> ParseResult<(i32, i32)> {
        let parts: Vec<_> = s.split(",").collect();

        if parts.len() != 2 {
            return Err(ConfigParseError::InvalidPosition {
                value: s.to_string(),
                reason: "expected format: X,Y (e.g., 1920,0)".to_string(),
            });
        }

        let x = parts[0]
            .trim()
            .parse::<i32>()
            .map_err(|_| ConfigParseError::InvalidPosition {
                value: s.to_string(),
                reason: format!("invalid X coordinate: {}", parts[0]),
            })?;

        let y = parts[1]
            .trim()
            .parse::<i32>()
            .map_err(|_| ConfigParseError::InvalidPosition {
                value: s.to_string(),
                reason: format!("invalid Y coordinate: {}", parts[1]),
            })?;

        Ok((x, y))
    }

    fn parse_scale(&mut self) -> ParseResult<OutputCommand> {
        assert!(self.check(&Token::Scale));
        self.advance();

        let s = if let Token::Number(scale_str) = &self.tokens[self.current] {
            *scale_str
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "scale number (e.g., 1.5)".to_string(),
                found: format!("{:?}", &self.tokens[self.current]),
                position: self.current,
            });
        };

        Ok(OutputCommand::Scale(s))
    }

    fn parse_transform(&mut self) -> ParseResult<OutputCommand> {
        assert!(self.check(&Token::Transform));
        self.advance();

        let transform = match &self.tokens[self.current] {
            Token::Number(n) => match *n as i32 {
                90 => Transform::Rotate90,
                180 => Transform::Rotate180,
                270 => Transform::Rotate270,
                _ => {
                    return Err(ConfigParseError::InvalidTransform {
                        value: n.to_string(),
                    });
                }
            },
            Token::Identifier(s) => match s.as_str() {
                "normal" => Transform::Normal,
                "flipped" => Transform::Flipped,
                "flipped-90" => Transform::Flipped90,
                "flipped-180" => Transform::Flipped180,
                "flipped-270" => Transform::Flipped270,
                _ => {
                    return Err(ConfigParseError::InvalidTransform { value: s.clone() });
                }
            },
            other => {
                return Err(ConfigParseError::UnexpectedToken {
                    expected: "transform value (normal, 90, 180, 270, flipped, flipped-90, etc.)"
                        .to_string(),
                    found: format!("{:?}", other),
                    position: self.current,
                });
            }
        };

        Ok(OutputCommand::Transform(transform))
    }

    fn parse_adaptive_sync(&mut self) -> ParseResult<OutputCommand> {
        assert!(self.check(&Token::AdaptiveSync));
        self.advance();

        let adaptive_sync = if let Token::Identifier(a) = &self.tokens[self.current] {
            match a.as_str() {
                "on" => OutputCommand::AdaptiveSync(true),
                "off" => OutputCommand::AdaptiveSync(false),
                _ => {
                    return Err(ConfigParseError::InvalidAdaptiveSync { value: a.clone() });
                }
            }
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "on or off".to_string(),
                found: format!("{:?}", &self.tokens[self.current]),
                position: self.current,
            });
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
