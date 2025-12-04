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

pub async fn parse_file(path: PathBuf) -> AppResult<Vec<Directive>> {
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
    let top_level_vec: Vec<Directive> = Vec::new();
    let result = recursive_read(text_for_parsing, top_level_vec);
    result
}

fn recursive_read(
    mut text: BTreeMap<usize, &str>,
    mut dir_vec: Vec<Directive>,
) -> AppResult<Vec<Directive>> {
    if text.len() == 0 {
        return Err(KanskeError::ParsedStringIsEmpty);
    } else if text.len() == 1 {
        let directive = Directive::from_line(text)?;
        dir_vec.push(directive);
        return Ok(dir_vec);
    } else {
        // --------------------
        // From here text == block
        // --------------------
        let (line_no, first_line) = text
            .pop_first()
            .ok_or_else(|| KanskeError::ParsedStringIsEmpty)?;
        let mut first_entry = BTreeMap::new();
        first_entry.insert(line_no, first_line);

        if first_line.contains("{") {
            let child_vec: Vec<Directive> = Vec::new();

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

            let mut directive = Directive::from_line(first_entry)?;
            let tail = text.split_off(&key);
            dbg!(&text, &tail);
            let child = recursive_read(text, child_vec)?;
            directive.children = Some(Box::new(child));
            dbg!(&directive);
            return Ok(dir_vec);
        } else {
            dbg!(first_line);
            let directive = Directive::from_line(first_entry)?;
            let mut next = recursive_read(text, dir_vec)?;
            next.push(directive);
            return Ok(next);
        }
    }
}
