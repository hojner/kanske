use std::{env, fs, path::PathBuf, str::FromStr, sync::Arc};

use crate::{
    AppResult, KanskeError,
    parser::{output_parser::types::Params, profile_parser::Profile},
};

pub struct Block {
    pub directives: Directive,
    pub directives_len: usize,
}

// pub enum Params {
//     Output(Params),
//     Profile(Profile),
// }

pub struct Directive {
    pub name: String,
    pub params: Params,
    pub params_len: usize,
    pub children: Option<Box<Directive>>,
    pub line_no: usize,
}

impl Block {
    fn from_output(line_no: usize, output_str: &str) -> AppResult<Self> {
        Ok(Self {
            directives: Directive {
                name: "output".to_string(),
                params: Params::from_str(output_str)?,
                params_len: 0,
                children: None,
                line_no,
            },
            directives_len: 0,
        })
    }
    // fn from_profile(line_no: usize, profile_str: &str) -> AppResult<Self> {
    //     let profile = Profile::from_str(profile_str)?;
    //     Ok(Self {
    //         directives: Directive {
    //             name: "profile".to_string(),
    //             params: Params::Profile(profile),
    //             params_len: 0,
    //             children: None,
    //             line_no,
    //         },
    //         directives_len: 0,
    //     })
    // }
}

// We keep track of where we are in the parsing chain, at the oplevel or "in a profile"
pub enum ParserState {
    Toplevel,
    InProfile(Arc<str>),
}

pub async fn parse_file(path: PathBuf) -> AppResult<Arc<[Directive]>> {
    let config_file = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(KanskeError::ReadIOError(e)),
    };
    let mut parser_state = ParserState::Toplevel;
    for (n, line) in config_file.lines().enumerate() {
        if line.starts_with("#") || line.is_empty() {
            continue;
        } else if line.starts_with("output") {
            Block::from_output(n, line);
        } else if line.starts_with("profile") {
            return Err(KanskeError::LimitedFunctionError(
                "profile blocks not yet supported".to_string(),
            ));
        } else {
            todo!()
        }
    }

    Ok(Arc::new([Directive {
        name: "test".to_string(),
        params: Params::new(),
        params_len: 1,
        children: None,
        line_no: 1,
    }]))
}

fn config_base() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|h| !h.is_empty())
                .map(|m| PathBuf::from(m).join(".config"))
        })
}

// fn line_parser(line: &str, state: ParserState) ->
