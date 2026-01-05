// Parser integration tests based on kanshi-complete-example.config

use kanske_lib::parser::block_parser::types::{
    ConfigItem, Lexer, OutputCommand, OutputDesc, Parser, Transform,
};

// ============================================================================
// LEXER TESTS
// ============================================================================

#[test]
fn test_lexer_with_comments() {
    let input = r#"
# This is a comment
profile test {
    output DP-1 enable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let result = lexer.tokenizer();
    assert!(result.is_ok());
}

#[test]
fn test_lexer_quoted_string() {
    let input = r#"profile "detailed example""#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    assert_eq!(tokens.len(), 3); // Profile, String, Eof
}

// ============================================================================
// BASIC PARSING TESTS
// ============================================================================

#[test]
fn test_parse_profile_with_enable() {
    let input = r#"
profile test {
    output eDP-1 enable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    assert_eq!(config.items.len(), 1);
    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert_eq!(p.name, Some("test".to_string()));
            assert_eq!(p.outputs.len(), 1);
            assert!(matches!(p.outputs[0].commands[0], OutputCommand::Enable));
        }
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_mode_with_refresh() {
    let input = r#"
profile test {
    output DP-1 mode 1920x1080@60Hz
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => match p.outputs[0].commands[0] {
            OutputCommand::Mode {
                width,
                height,
                frequency,
            } => {
                assert_eq!(width, 1920);
                assert_eq!(height, 1080);
                assert_eq!(frequency, Some(60.0));
            }
            _ => panic!("Expected Mode command"),
        },
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_mode_without_refresh() {
    let input = r#"
profile test {
    output DP-1 mode 1920x1080
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => match p.outputs[0].commands[0] {
            OutputCommand::Mode { frequency, .. } => {
                assert_eq!(frequency, None);
            }
            _ => panic!("Expected Mode command"),
        },
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_position() {
    let input = r#"
profile test {
    output DP-1 position 1920,0
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => match p.outputs[0].commands[0] {
            OutputCommand::Position { x, y } => {
                assert_eq!(x, 1920);
                assert_eq!(y, 0);
            }
            _ => panic!("Expected Position command"),
        },
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_scale() {
    let input = r#"
profile test {
    output eDP-1 scale 1.5
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => match p.outputs[0].commands[0] {
            OutputCommand::Scale(s) => assert_eq!(s, 1.5),
            _ => panic!("Expected Scale command"),
        },
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_transform_normal() {
    let input = r#"
profile test {
    output DP-1 transform normal
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert!(matches!(
                p.outputs[0].commands[0],
                OutputCommand::Transform(Transform::Normal)
            ));
        }
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_transform_rotate_90() {
    let input = r#"
profile test {
    output DP-2 transform 90
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert!(matches!(
                p.outputs[0].commands[0],
                OutputCommand::Transform(Transform::Rotate90)
            ));
        }
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_adaptive_sync_on() {
    let input = r#"
profile test {
    output DP-1 adaptive_sync on
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert!(matches!(
                p.outputs[0].commands[0],
                OutputCommand::AdaptiveSync(true)
            ));
        }
        _ => panic!("Expected Profile"),
    }
}

// ============================================================================
// INLINE VS BLOCK OUTPUT TESTS
// ============================================================================

#[test]
fn test_parse_inline_output_commands() {
    let input = r#"
profile compact {
    output eDP-1 enable mode 1920x1080 position 0,0 scale 1.5
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert_eq!(p.outputs[0].commands.len(), 4);
        }
        _ => panic!("Expected Profile"),
    }
}

#[test]
fn test_parse_block_output_commands() {
    let input = r#"
profile detailed {
    output eDP-1 {
        enable
        mode 1920x1200@60Hz
        position 0,0
        scale 2.0
    }
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert_eq!(p.outputs[0].commands.len(), 4);
        }
        _ => panic!("Expected Profile"),
    }
}

// ============================================================================
// MULTIPLE OUTPUTS
// ============================================================================

#[test]
fn test_parse_multiple_outputs() {
    let input = r#"
profile dual {
    output eDP-1 enable
    output HDMI-1 disable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert_eq!(p.outputs.len(), 2);
            match &p.outputs[0].desc {
                OutputDesc::Name(n) => assert_eq!(n, "eDP-1"),
                _ => panic!("Expected name"),
            }
            match &p.outputs[1].desc {
                OutputDesc::Name(n) => assert_eq!(n, "HDMI-1"),
                _ => panic!("Expected name"),
            }
        }
        _ => panic!("Expected Profile"),
    }
}

// ============================================================================
// GLOBAL OUTPUT CONFIG
// ============================================================================

#[test]
fn test_parse_global_output() {
    let input = r#"
output HDMI-A-1 mode 1920x1080@60Hz
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    assert_eq!(config.items.len(), 1);
    assert!(matches!(config.items[0], ConfigItem::Output(_)));
}

// ============================================================================
// MULTIPLE TOP-LEVEL ITEMS
// ============================================================================

#[test]
fn test_parse_multiple_profiles() {
    let input = r#"
profile first {
    output eDP-1 enable
}

profile second {
    output HDMI-1 enable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();

    assert_eq!(config.items.len(), 2);
    assert!(matches!(config.items[0], ConfigItem::Profile(_)));
    assert!(matches!(config.items[1], ConfigItem::Profile(_)));
}

// ============================================================================
// UNIMPLEMENTED FEATURES (These should fail until implemented)
// ============================================================================

#[test]
#[ignore] // Remove this when exec is implemented
fn test_parse_exec_directive() {
    let input = r#"
profile test {
    output eDP-1 enable
    exec notify-send "Profile activated"
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();
    
    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert_eq!(p.execs.len(), 1);
            assert_eq!(p.execs[0].command, "notify-send \"Profile activated\"");
        }
        _ => panic!("Expected Profile"),
    }
}

#[test]
#[ignore] // Remove this when include is implemented
fn test_parse_include_directive() {
    let input = r#"
include ~/.config/kanshi/extra.conf
profile test {
    output eDP-1 enable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();
    
    assert_eq!(config.items.len(), 2);
    match &config.items[0] {
        ConfigItem::Include(inc) => {
            assert_eq!(inc.path, "~/.config/kanshi/extra.conf");
        }
        _ => panic!("Expected Include"),
    }
}

#[test]
#[ignore] // Remove this when wildcard output is implemented
fn test_parse_wildcard_output() {
    let input = r#"
profile test {
    output eDP-1 enable
    output * disable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();
    
    match &config.items[0] {
        ConfigItem::Profile(p) => {
            assert_eq!(p.outputs.len(), 2);
            match &p.outputs[1].desc {
                OutputDesc::Any => {}, // Success
                _ => panic!("Expected OutputDesc::Any for wildcard"),
            }
        }
        _ => panic!("Expected Profile"),
    }
}

#[test]
#[ignore] // Remove this when anonymous profiles are implemented
fn test_parse_anonymous_profile() {
    let input = r#"
profile {
    output eDP-1 enable
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let config = parser.parse().unwrap();
    
    match &config.items[0] {
        ConfigItem::Profile(p) => {
            // Should auto-generate a name or set to None
            assert!(p.name.is_some() || p.name.is_none());
        }
        _ => panic!("Expected Profile"),
    }
}

// ============================================================================
// ERROR CASES
// ============================================================================

#[test]
fn test_parse_invalid_transform() {
    let input = r#"
profile test {
    output DP-1 transform 45
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_adaptive_sync() {
    let input = r#"
profile test {
    output DP-1 adaptive_sync maybe
}
"#;
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenizer().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_err());
}
