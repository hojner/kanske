use kanske_lib::parser::block_parser::parse_file;
use kanske_lib::parser::block_parser::types::Directive;
use std::path::PathBuf;

// Helper to create temporary test config files
async fn test_config_parse(content: &str) -> Result<Directive, kanske_lib::KanskeError> {
    use std::io::Write;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    let path = temp_file.path().to_path_buf();
    let result = parse_file(path).await;
    result
}

// Helper to check if directive has expected name
fn assert_directive_name(directive: &Directive, expected_name: &str) {
    assert_eq!(
        directive.name.as_ref(),
        expected_name,
        "Expected directive name '{}', got '{}'",
        expected_name,
        directive.name
    );
}

// Helper to check if mode is parsed correctly
fn assert_mode(
    directive: &Directive,
    expected_width: u32,
    expected_height: u32,
    expected_freq: Option<f32>,
) {
    let mode = directive
        .params
        .mode
        .as_ref()
        .expect("Mode should be present");
    assert_eq!(mode.width, expected_width, "Width mismatch");
    assert_eq!(mode.height, expected_height, "Height mismatch");
    assert_eq!(mode.frequency, expected_freq, "Frequency mismatch");
}

// ============================================================================
// POSITIVE TESTS - Valid Configurations with Value Assertions
// ============================================================================

#[tokio::test]
async fn test_simple_mode_parsing() {
    let config = r#"
output DP-1 mode 1920x1080@60Hz
"#;
    let result = test_config_parse(config).await;

    match result {
        Ok(directive) => {
            assert_directive_name(&directive, "output");
            assert_mode(&directive, 1920, 1080, Some(60.0));
        }
        Err(e) => {
            panic!("Should parse simple mode directive. Error: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_mode_high_refresh_rate() {
    let config = "output DP-1 mode 2560x1440@165Hz";
    let result = test_config_parse(config).await;

    if let Ok(directive) = result {
        assert_mode(&directive, 2560, 1440, Some(165.0));
    } else {
        panic!("Should parse high refresh rate. Error: {:?}", result);
    }
}

#[tokio::test]
async fn test_mode_4k_resolution() {
    let config = "output HDMI-1 mode 3840x2160@60Hz";
    let result = test_config_parse(config).await;

    if let Ok(directive) = result {
        assert_mode(&directive, 3840, 2160, Some(60.0));
    }
}

#[tokio::test]
async fn test_mode_fractional_refresh_rate() {
    let config = "output eDP-1 mode 1920x1080@59.94Hz";
    let result = test_config_parse(config).await;

    if let Ok(directive) = result {
        assert_mode(&directive, 1920, 1080, Some(59.94));
    }
}

// ============================================================================
// NEGATIVE TESTS - Invalid Configurations
// ============================================================================

#[tokio::test]
async fn test_invalid_mode_missing_x_separator() {
    let config = "output DP-1 mode 1920-1080@60Hz";
    let result = test_config_parse(config).await;

    assert!(
        result.is_err(),
        "Should fail: mode format missing 'x' separator"
    );
}

#[tokio::test]
async fn test_invalid_mode_non_numeric_width() {
    let config = "output DP-1 mode ABCDx1080@60Hz";
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: width is not a number");
}

#[tokio::test]
async fn test_invalid_mode_non_numeric_height() {
    let config = "output DP-1 mode 1920xABCD@60Hz";
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: height is not a number");
}

#[tokio::test]
async fn test_invalid_mode_non_numeric_refresh() {
    let config = "output DP-1 mode 1920x1080@ABCHz";
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: refresh rate is not a number");
}

#[tokio::test]
async fn test_mismatched_braces_extra_open() {
    let config = r#"
profile test {
    output eDP-1 mode 1920x1080
    {
"#;
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: mismatched opening braces");
}

#[tokio::test]
async fn test_mismatched_braces_extra_close() {
    let config = r#"
profile test {
    output eDP-1 mode 1920x1080
}
}
"#;
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: extra closing braces");
}

#[tokio::test]
async fn test_multiple_braces_same_line() {
    let config = "profile test { { output eDP-1 mode 1920x1080 } }";
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: multiple braces on same line");
}

#[tokio::test]
async fn test_empty_config_file() {
    let config = "";
    let result = test_config_parse(config).await;

    assert!(result.is_err(), "Should fail: empty config file");
}
