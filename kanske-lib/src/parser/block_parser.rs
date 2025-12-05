pub mod types;

use crate::{AppResult, KanskeError, parser::block_parser::types::Directive};
use std::{collections::BTreeMap, fs, path::PathBuf};

pub async fn parse_file(path: PathBuf) -> AppResult<Vec<Directive>> {
    let config_file = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(KanskeError::ReadIOError(e)),
    };
    let text_for_parsing = config_file
        .lines()
        .by_ref()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.starts_with("#") && !l.is_empty())
        .collect::<BTreeMap<usize, &str>>();
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
    let result = recursive_read(text_for_parsing, Vec::new());
    result
}

fn find_matching_brace(text: &BTreeMap<usize, &str>) -> AppResult<usize> {
    let mut depth = 1;
    for (i, l) in text.iter() {
        let open_count = l.matches("{").count();
        let close_count = l.matches("}").count();

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

fn recursive_read(
    mut text: BTreeMap<usize, &str>,
    mut dir_vec: Vec<Directive>,
) -> AppResult<Vec<Directive>> {
    if text.is_empty() {
        return Ok(dir_vec);
    }

    if text.len() == 1 {
        let directive = Directive::from_line(text)?;
        dir_vec.push(directive);
        return Ok(dir_vec);
    }

    let (line_no, first_line) = text
        .pop_first()
        .ok_or_else(|| KanskeError::ParsedStringIsEmpty)?;
    let mut first_entry = BTreeMap::new();
    first_entry.insert(line_no, first_line);

    if first_line.contains("{") {
        let key = find_matching_brace(&text)?;
        let mut directive = Directive::from_line(first_entry)?;
        let mut tail = text.split_off(&key);
        tail.pop_first();

        let child = recursive_read(text, Vec::new())?;
        directive.children = Some(Box::new(child));
        dir_vec.push(directive);

        recursive_read(tail, dir_vec)
    } else {
        let directive = Directive::from_line(first_entry)?;
        dir_vec.push(directive);
        recursive_read(text, dir_vec)
    }
}
