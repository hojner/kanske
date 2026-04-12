use tracing::{debug, info, warn};

use crate::composer::compose_profiles;
use crate::error::{AppResult, ConfigParseError};
use crate::parser::ast::{Config, ConfigItem};
use crate::parser::{lexer::Lexer, parse::Parser};

use std::{fs, path::PathBuf};

const MAX_INCLUDE_DEPTH: usize = 10;

pub fn parse_file(path: PathBuf) -> AppResult<Config> {
    let config = parse_file_recursive(&path, 0)?;

    let profiles = config
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Profile(_)))
        .count();
    let global_outputs = config
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Output(_)))
        .count();
    info!(path = %path.display(), profiles, global_outputs, "Config loaded");

    let result = compose_profiles(config)?;
    Ok(result)
}

fn parse_file_recursive(path: &PathBuf, depth: usize) -> AppResult<Config> {
    let path_str = path.display().to_string();

    if depth > MAX_INCLUDE_DEPTH {
        return Err(ConfigParseError::IncludeDepthExceeded {
            path: path_str.clone(),
        }
        .into_config_error(path_str));
    }

    check_file_permissions(path);

    let config_file = fs::read_to_string(path)?;

    let mut lexer = Lexer::new(config_file);
    let tokens = lexer
        .tokenizer()
        .map_err(|e| e.into_config_error(path_str.clone()))?;
    debug!(path = %path_str, token_count = tokens.len(), "Lexer complete");

    let mut ast = Parser::new(tokens);
    let parse_result = ast
        .parse()
        .map_err(|e| e.into_config_error(path_str.clone()))?;

    let base_dir = path.parent().unwrap_or(path);
    let mut resolved_items: Vec<ConfigItem> = Vec::new();

    for item in parse_result.items {
        match item {
            ConfigItem::Include(ref inc) => {
                let expanded = expand_tilde(&inc.path);
                let pattern = if PathBuf::from(&expanded).is_relative() {
                    base_dir.join(&expanded).display().to_string()
                } else {
                    expanded
                };

                let paths = glob::glob(&pattern).map_err(|e| {
                    ConfigParseError::IncludeError {
                        path: inc.path.clone(),
                        reason: format!("invalid glob pattern: {}", e),
                    }
                    .into_config_error(path_str.clone())
                })?;

                let mut matched_any = false;
                for entry in paths {
                    let entry_path = entry.map_err(|e| {
                        ConfigParseError::IncludeError {
                            path: inc.path.clone(),
                            reason: format!("glob error: {}", e),
                        }
                        .into_config_error(path_str.clone())
                    })?;
                    debug!(include = %entry_path.display(), depth, "Processing include");
                    let included = parse_file_recursive(&entry_path, depth + 1)?;
                    resolved_items.extend(included.items);
                    matched_any = true;
                }
                if !matched_any {
                    warn!(pattern = %inc.path, "Include pattern matched no files");
                }
            }
            other => resolved_items.push(other),
        }
    }

    Ok(Config {
        items: resolved_items,
    })
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest).display().to_string();
    }
    path.to_string()
}

fn check_file_permissions(path: &PathBuf) {
    use std::os::unix::fs::MetadataExt;

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };

    let current_uid = nix::unistd::Uid::current().as_raw();
    let file_uid = metadata.uid();
    let mode = metadata.mode();
    let path_display = path.display().to_string();
    let mut warnings: Vec<String> = Vec::new();

    if file_uid != current_uid && file_uid != 0 {
        let msg = format!(
            "Config file '{}' is not owned by current user or root (owner uid: {})",
            path_display, file_uid
        );
        warn!(
            path = %path_display,
            file_owner = file_uid,
            current_user = current_uid,
            "Config file is not owned by current user or root"
        );
        warnings.push(msg);
    }
    if mode & 0o002 != 0 {
        let msg = format!("Config file '{}' is world-writable", path_display);
        warn!(path = %path_display, "Config file is world-writable");
        warnings.push(msg);
    }
    if mode & 0o020 != 0 {
        let msg = format!("Config file '{}' is group-writable", path_display);
        warn!(path = %path_display, "Config file is group-writable");
        warnings.push(msg);
    }

    if !warnings.is_empty() {
        let body = warnings.join("\n");
        if let Err(e) = std::process::Command::new("notify-send")
            .arg("--urgency=critical")
            .arg("--app-name=kanske")
            .arg("Kanske: Insecure config file")
            .arg(&body)
            .spawn()
        {
            debug!("Could not send desktop notification: {}", e);
        }
    }
}
