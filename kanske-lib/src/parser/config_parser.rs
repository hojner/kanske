use crate::error::{AppResult, KanskeError};
use crate::parser::{lexer::Lexer, parse::Parser};

use std::{fs, path::PathBuf};

pub async fn parse_file(path: PathBuf) -> AppResult<()> {
    let path_str = path.display().to_string();

    let config_file = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return Err(KanskeError::ReadIOError(e)),
    };

    let mut lexer = Lexer::new(config_file);
    let tokens = lexer
        .tokenizer()
        .map_err(|e| e.into_config_error(path_str.clone()))?;

    let mut ast = Parser::new(tokens);
    ast.parse().map_err(|e| e.into_config_error(path_str))?;

    todo!()
}
