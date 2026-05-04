use std::mem::discriminant;

use crate::error::{ConfigParseError, ParseResult};
use crate::parser::ast::{
    Config, ConfigItem, ExecDirective, IncludeDirective, OutputCommand, OutputConfig, OutputDesc,
    Profile, Transform,
};
use crate::parser::token::{Token, TokenHolder};

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: Vec<TokenHolder>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenHolder>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Config> {
        let mut config = Config { items: Vec::new() };
        let mut config_item;

        while !self.is_at_end() {
            config_item = match &self.tokens[self.current].token {
                Token::Profile => self.parse_profile()?,
                Token::Include => ConfigItem::Include(self.parse_include()?),
                Token::Output => ConfigItem::Output(self.parse_output()?),
                Token::Eof => break,
                other => {
                    return Err(ConfigParseError::UnexpectedToken {
                        expected: "profile or output".to_string(),
                        found: format!("{:?}", other),
                        position: self.tokens[self.current].position.clone(),
                    });
                }
            };
            config.items.push(config_item);
        }
        Ok(config)
    }

    fn parse_profile(&mut self) -> ParseResult<ConfigItem> {
        self.validate(&Token::Profile)?;
        self.advance();

        // TODO:
        // For now, a profile must have a name,
        // will handle name generation for anonymous profiles later
        //

        let name = match &self.tokens[self.current].token {
            Token::String(s) | Token::Identifier(s) => s.clone(),
            _ => {
                return Err(ConfigParseError::MissingProfileName {
                    position: self.tokens[self.current].position.clone(),
                });
            }
        };
        let mut profile = Profile::new(name);
        self.advance();
        self.validate(&Token::LeftBrace)?;
        self.advance();

        while !self.is_at_end() {
            match &self.tokens[self.current].token {
                Token::Output => {
                    profile.outputs.push(self.parse_output()?);
                }
                Token::Exec => {
                    profile.execs.push(self.parse_exec()?);
                }
                Token::RightBrace => {
                    self.advance();
                    break;
                }
                other => {
                    return Err(ConfigParseError::UnexpectedToken {
                        expected: "output, exec, or }".to_string(),
                        found: format!("{:?}", other),
                        position: self.tokens[self.current].position.clone(),
                    });
                }
            };
        }
        Ok(ConfigItem::Profile(profile))
    }

    fn parse_output(&mut self) -> ParseResult<OutputConfig> {
        self.validate(&Token::Output)?;
        self.advance();

        let desc = if let Token::Identifier(desc) = &self.tokens[self.current].token {
            match desc.as_str() {
                "*" => OutputDesc::Any,
                _ => OutputDesc::Name(desc.clone()),
            }
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "output name".to_string(),
                found: format!("{:?}", &self.tokens[self.current].token),
                position: self.tokens[self.current].position.clone(),
            });
        };

        self.advance();

        let mut commands = Vec::new();
        loop {
            match &self.tokens[self.current].token {
                Token::LeftBrace => {
                    self.advance();

                    while !self.is_at_end() && !self.check(&Token::RightBrace) {
                        commands.push(self.parse_output_command()?);
                        self.advance();
                    }
                    self.validate(&Token::RightBrace)?;
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
                Token::Enabled(_)
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
                        position: self.tokens[self.current].position.clone(),
                    });
                }
            };
        }

        Ok(OutputConfig { desc, commands })
    }

    fn parse_output_command(&mut self) -> ParseResult<OutputCommand> {
        match &self.tokens[self.current].token {
            Token::Enabled(b) => Ok(OutputCommand::Enabled(*b)),
            Token::Mode => self.parse_mode(),
            Token::Position => self.parse_position(),
            Token::Scale => self.parse_scale(),
            Token::Transform => self.parse_transform(),
            Token::AdaptiveSync => self.parse_adaptive_sync(),
            other => Err(ConfigParseError::UnexpectedToken {
                expected: "output command".to_string(),
                found: format!("{:?}", other),
                position: self.tokens[self.current].position.clone(),
            }),
        }
    }

    fn parse_mode(&mut self) -> ParseResult<OutputCommand> {
        self.validate(&Token::Mode)?;
        self.advance();

        let (width, height, frequency) =
            if let Token::Identifier(mode_str) = &self.tokens[self.current].token {
                self.parse_mode_str(mode_str)?
            } else {
                return Err(ConfigParseError::UnexpectedToken {
                    expected: "mode string (e.g., 1920x1080@60Hz)".to_string(),
                    found: format!("{:?}", &self.tokens[self.current].token),
                    position: self.tokens[self.current].position.clone(),
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
        self.validate(&Token::Position)?;
        self.advance();

        let (x, y) = if let Token::Identifier(position_str) = &self.tokens[self.current].token {
            self.parse_position_str(position_str)?
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "position string (e.g., 1920,0)".to_string(),
                found: format!("{:?}", &self.tokens[self.current]),
                position: self.tokens[self.current].position.clone(),
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
        self.validate(&Token::Scale)?;
        self.advance();

        let s = if let Token::Number(scale_str) = &self.tokens[self.current].token {
            *scale_str
        } else {
            return Err(ConfigParseError::UnexpectedToken {
                expected: "scale number (e.g., 1.5)".to_string(),
                found: format!("{:?}", &self.tokens[self.current]),
                position: self.tokens[self.current].position.clone(),
            });
        };

        Ok(OutputCommand::Scale(s))
    }

    fn parse_transform(&mut self) -> ParseResult<OutputCommand> {
        self.validate(&Token::Transform)?;
        self.advance();

        let transform = match &self.tokens[self.current].token {
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
                    position: self.tokens[self.current].position.clone(),
                });
            }
        };

        Ok(OutputCommand::Transform(transform))
    }

    fn parse_adaptive_sync(&mut self) -> ParseResult<OutputCommand> {
        self.validate(&Token::AdaptiveSync)?;
        self.advance();

        let adaptive_sync = if let Token::Identifier(a) = &self.tokens[self.current].token {
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
                position: self.tokens[self.current].position.clone(),
            });
        };

        Ok(adaptive_sync)
    }

    fn check(&self, token: &Token) -> bool {
        discriminant(&self.tokens[self.current].token) == discriminant(token)
    }

    fn validate(&self, token: &Token) -> ParseResult<()> {
        if !self.check(token) {
            return Err(ConfigParseError::UnexpectedToken {
                expected: format!("{:?}", token),
                found: format!("{:?}", self.tokens[self.current]),
                position: self.tokens[self.current].position.clone(),
            });
        }
        Ok(())
    }

    fn parse_exec(&mut self) -> ParseResult<ExecDirective> {
        self.validate(&Token::Exec)?;
        self.advance();

        let command = match &self.tokens[self.current].token {
            Token::String(s) => s.clone(),
            other => {
                return Err(ConfigParseError::UnexpectedToken {
                    expected: "exec command string".to_string(),
                    found: format!("{:?}", other),
                    position: self.tokens[self.current].position.clone(),
                });
            }
        };
        self.advance();

        Ok(ExecDirective { command })
    }

    fn parse_include(&mut self) -> ParseResult<IncludeDirective> {
        self.validate(&Token::Include)?;
        self.advance();

        let path = match &self.tokens[self.current].token {
            Token::String(s) => s.clone(),
            other => {
                return Err(ConfigParseError::UnexpectedToken {
                    expected: "include path".to_string(),
                    found: format!("{:?}", other),
                    position: self.tokens[self.current].position.clone(),
                });
            }
        };
        self.advance();

        Ok(IncludeDirective { path })
    }

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
