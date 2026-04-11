use tracing::{debug, info};

use crate::composer::compose_profiles;
use crate::error::AppResult;
use crate::parser::ast::{Config, ConfigItem};
use crate::parser::{lexer::Lexer, parse::Parser};

use std::{fs, path::PathBuf};

pub fn parse_file(path: PathBuf) -> AppResult<Config> {
    let path_str = path.display().to_string();
    let config_file = fs::read_to_string(&path)?;

    let mut lexer = Lexer::new(config_file);
    let tokens = lexer
        .tokenizer()
        .map_err(|e| e.into_config_error(path_str.clone()))?;
    debug!(token_count = tokens.len(), "Lexer complete");

    let mut ast = Parser::new(tokens);
    let parse_result = ast
        .parse()
        .map_err(|e| e.into_config_error(path_str.clone()))?;

    let profiles = parse_result
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Profile(_)))
        .count();
    let global_outputs = parse_result
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Output(_)))
        .count();
    info!(path = %path_str, profiles, global_outputs, "Config loaded");

    let result = compose_profiles(parse_result)?;
    Ok(result)
}
