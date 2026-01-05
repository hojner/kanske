pub mod types;

use crate::{
    AppResult, KanskeError,
    parser::block_parser::types::{Lexer, Parser},
};
use std::{fs, path::PathBuf};

pub async fn parse_file(path: PathBuf) -> AppResult<()> {
    let config_file = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(KanskeError::ReadIOError(e)),
    };

    let mut lexer = Lexer::new(config_file);
    let tokens = lexer.tokenizer()?;

    let mut ast = Parser::new(tokens);
    ast.parse()?;

    // dbg!(&tokens);

    todo!()
}
