pub mod types;

use std::{collections::BTreeMap, fs, io::Lines, path::PathBuf, sync::Arc};

use crate::{
    AppResult, KanskeError,
    parser::{
        block_parser::types::{Directive, Params},
        profile_parser::Profile,
    },
};

pub enum ParserState {
    Toplevel,
    InProfile(Arc<str>),
}

pub async fn parse_file(path: PathBuf) -> AppResult<Directive> {
    let config_file = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(KanskeError::ReadIOError(e)),
    };
    let text_for_parsing = config_file
        .lines()
        .by_ref()
        .enumerate()
        .filter(|(_, l)| !l.starts_with("#") && !l.is_empty())
        .collect::<BTreeMap<usize, &str>>();
    dbg!(&text_for_parsing);
    let (open, close) = text_for_parsing.values().fold((0, 0), |(open, close), s| {
        (
            open + s.matches('{').count(),
            close + s.matches('}').count(),
        )
    });
    if open != close {
        return Err(KanskeError::ParsedStringUnexpectedFormat(
            "The number of { and } does not match".to_string(),
        ));
    }
    let result = recursive_read(text_for_parsing);
    result
}

fn recursive_read(mut text: BTreeMap<usize, &str>) -> AppResult<Directive> {
    if text.len() == 0 {
        return Err(KanskeError::ParsedStringIsEmpty);
    } else if text.len() == 1 {
        // todo!("one-line parsing not implemented")
        return Directive::from_line(text);
    } else {
        let (line_no, first_line) = text
            .pop_first()
            .ok_or_else(|| KanskeError::ParsedStringIsEmpty)?;
        if first_line.contains("{") {
            // ------------------
            // Block för depth-checking and block creation
            // ------------------
            let mut depth = 1;
            let mut key: usize = 0;
            let mut block = text.iter();
            while depth > 0 {
                let (i, e) = match block.next() {
                    Some((i, e)) => (i, e),
                    None => {
                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                            "Curly braces not matching".to_string(),
                        ));
                    }
                };
                if e.matches("{").count() > 1 || e.matches("}").count() > 1 {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "Multiple { or } cannot be on the same line".to_string(),
                    ));
                } else if e.contains("{") && e.contains("}") {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "{ and } cannot be on the same line".to_string(),
                    ));
                } else if e.contains("{") {
                    depth += 1;
                } else if e.contains("}") {
                    depth -= 1;
                    key = *i;
                }
            }
            // -----------------
            // End of block
            // -----------------
            let rest_of_text = text.split_off(&key);
            dbg!(&text, &rest_of_text);
            let child = recursive_read(text);
            dbg!(&child);
        } else {
            dbg!(first_line);
            // text.pop_first();
            let next = recursive_read(text);
        }
    }
    todo!("recursive read not yet implemented")
    // Ok(Directive::from_line(text)?)
}
