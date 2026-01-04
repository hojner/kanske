pub mod types;

use crate::{
    AppResult, KanskeError,
    parser::block_parser::types::{Directive, Lexer, Parser},
};
use std::{collections::BTreeMap, fs, path::PathBuf};

pub async fn parse_file(path: PathBuf) -> AppResult<Vec<Directive>> {
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

fn find_matching_brace(text: &BTreeMap<usize, &str>) -> AppResult<usize> {
    let mut depth = 1;
    for (i, l) in text.iter() {
        let open_count = l.matches("{").count();
        let close_count = l.matches("}").count();

        dbg!(&open_count, &close_count);

        if open_count > 1 || close_count > 1 {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "Multiple { or } cannot be on the same line. ".to_string(),
            ));
        }
        if l.contains("{") && l.contains("}") {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "{ and } cannot be on the same line. ".to_string(),
            ));
        }

        depth += open_count;
        depth -= close_count;

        if depth == 0 {
            return Ok(*i);
        }
    }
    Err(KanskeError::ParsedStringUnexpectedFormat(
        "Curly braces not matching in config".to_string(),
    ))
}
