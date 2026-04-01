use crate::composer::compose_profiles;
use crate::error::AppResult;
use crate::parser::ast::Config;
use crate::parser::{lexer::Lexer, parse::Parser};

use std::{fs, path::PathBuf};

pub fn parse_file(path: PathBuf) -> AppResult<Config> {
    let path_str = path.display().to_string();
    let config_file = fs::read_to_string(&path)?;

    let mut lexer = Lexer::new(config_file);
    let tokens = lexer
        .tokenizer()
        .map_err(|e| e.into_config_error(path_str.clone()))?;

    let mut ast = Parser::new(tokens);
    let parse_result = ast
        .parse()
        .map_err(|e| e.into_config_error(path_str.clone()))?;
    let result = compose_profiles(parse_result)?;
    Ok(result)
}
