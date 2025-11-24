pub mod from_str_impls;
pub mod types;

use std::{fs, io::Lines, path::PathBuf, sync::Arc};

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

fn recursive_read(line_no: usize, text: &str) -> AppResult<Directive> {
    if text.lines().count() == 1 {
        return Directive::from_line(line_no, text);
    } else {
        if text.contains("{") {
            let trimmed = text
                .lines()
                .enumerate()
                .filter(|(i, _)| *i != 0 && *i != text.lines().count() - 1)
                .map(|(_, line)| line)
                .collect::<Vec<_>>()
                .join("\n");
            let text = recursive_read(1, &trimmed);
            println!("does contain {{: {:?}", text);
        } else {
            let mut dir_vec: Vec<_> = vec![];
            for line in text.lines() {
                let dir = recursive_read(1, line)?;
                dir_vec.push(dir);
                dbg!(&dir_vec);
            }
            println!("does not contain {{: {:?}", text);
        }
        return Directive::from_line(1, "line");
    };
}

pub async fn parse_file(path: PathBuf) -> AppResult<Directive> {
    let config_file = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(KanskeError::ReadIOError(e)),
    };
    let result = recursive_read(1, &config_file);
    result
}
