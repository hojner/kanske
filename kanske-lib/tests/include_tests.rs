// Integration tests for include directive resolution

use kanske_lib::parser::{ast::ConfigItem, config_parser::parse_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_include_single_file() {
    let dir = TempDir::new().unwrap();

    let extra = dir.path().join("extra.conf");
    fs::write(
        &extra,
        r#"
profile included {
    output HDMI-1 enable
}
"#,
    )
    .unwrap();

    let main_config = dir.path().join("config");
    fs::write(
        &main_config,
        format!(
            r#"
include {}
profile main {{
    output eDP-1 enable
}}
"#,
            extra.display()
        ),
    )
    .unwrap();

    let config = parse_file(main_config).unwrap();
    let profiles: Vec<_> = config
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Profile(_)))
        .collect();
    assert_eq!(profiles.len(), 2);
}

#[test]
fn test_include_glob_pattern() {
    let dir = TempDir::new().unwrap();
    let conf_dir = dir.path().join("conf.d");
    fs::create_dir(&conf_dir).unwrap();

    fs::write(
        conf_dir.join("a.conf"),
        r#"
profile a {
    output DP-1 enable
}
"#,
    )
    .unwrap();

    fs::write(
        conf_dir.join("b.conf"),
        r#"
profile b {
    output DP-2 enable
}
"#,
    )
    .unwrap();

    let main_config = dir.path().join("config");
    fs::write(
        &main_config,
        format!(
            r#"
include {}/*.conf
"#,
            conf_dir.display()
        ),
    )
    .unwrap();

    let config = parse_file(main_config).unwrap();
    let profiles: Vec<_> = config
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Profile(_)))
        .collect();
    assert_eq!(profiles.len(), 2);
}

#[test]
fn test_include_nested() {
    let dir = TempDir::new().unwrap();

    let deep = dir.path().join("deep.conf");
    fs::write(
        &deep,
        r#"
profile deep {
    output DP-3 enable
}
"#,
    )
    .unwrap();

    let mid = dir.path().join("mid.conf");
    fs::write(
        &mid,
        format!(
            r#"
include {}
profile mid {{
    output DP-2 enable
}}
"#,
            deep.display()
        ),
    )
    .unwrap();

    let main_config = dir.path().join("config");
    fs::write(
        &main_config,
        format!(
            r#"
include {}
profile top {{
    output eDP-1 enable
}}
"#,
            mid.display()
        ),
    )
    .unwrap();

    let config = parse_file(main_config).unwrap();
    let profiles: Vec<_> = config
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Profile(_)))
        .collect();
    assert_eq!(profiles.len(), 3);
}

#[test]
fn test_include_depth_exceeded() {
    let dir = TempDir::new().unwrap();

    // Create a file that includes itself — should hit depth limit
    let self_include = dir.path().join("loop.conf");
    fs::write(
        &self_include,
        format!("include {}\n", self_include.display()),
    )
    .unwrap();

    let result = parse_file(self_include);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("depth limit exceeded"),
        "Expected depth limit error, got: {}",
        err
    );
}

#[test]
fn test_include_missing_file() {
    let dir = TempDir::new().unwrap();

    let main_config = dir.path().join("config");
    fs::write(
        &main_config,
        r#"
include /nonexistent/path/that/does/not/exist.conf
profile test {
    output eDP-1 enable
}
"#,
    )
    .unwrap();

    // Missing glob matches produce a warning but don't error
    let config = parse_file(main_config).unwrap();
    let profiles: Vec<_> = config
        .items
        .iter()
        .filter(|i| matches!(i, ConfigItem::Profile(_)))
        .collect();
    assert_eq!(profiles.len(), 1);
}

#[test]
fn test_include_with_global_outputs() {
    let dir = TempDir::new().unwrap();

    let extra = dir.path().join("defaults.conf");
    fs::write(
        &extra,
        r#"
output eDP-1 scale 2.0
"#,
    )
    .unwrap();

    let main_config = dir.path().join("config");
    fs::write(
        &main_config,
        format!(
            r#"
include {}
profile test {{
    output eDP-1 enable
}}
"#,
            extra.display()
        ),
    )
    .unwrap();

    let config = parse_file(main_config).unwrap();
    // After composition, the profile should have the global output merged
    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert!(p.outputs[0].commands.len() >= 2, "Expected merged commands");
        }
        _ => panic!("Expected Profile after composition"),
    }
}
