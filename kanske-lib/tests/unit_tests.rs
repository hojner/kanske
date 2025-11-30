// Unit tests for individual parser components

use kanske_lib::parser::block_parser::types::Mode;
use kanske_lib::KanskeError;

// ============================================================================
// MODE PARSING TESTS
// ============================================================================

#[test]
fn test_mode_with_refresh_and_hz() {
    // Mode needs to be public to test, or we need a public API
    // For now, this test shows the expected behavior
    // You may need to expose Mode::from_line as pub or use #[cfg(test)] pub
    
    // Example of what we want to test:
    // let mode = Mode::from_line("1920x1080@60Hz").unwrap();
    // assert_eq!(mode.width, 1920);
    // assert_eq!(mode.height, 1080);
    // assert_eq!(mode.frequency, Some(60.0));
}

#[test]
fn test_mode_with_refresh_no_hz() {
    let result = Mode::from_line("1920x1080@60");
    // This should work based on current implementation
    assert!(result.is_ok());
}

#[test]
fn test_mode_without_refresh() {
    let result = Mode::from_line("1920x1080");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 1920);
    assert_eq!(mode.height, 1080);
    assert_eq!(mode.frequency, None);
}

#[test]
fn test_mode_high_refresh_rate() {
    let result = Mode::from_line("2560x1440@165Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 2560);
    assert_eq!(mode.height, 1440);
    assert_eq!(mode.frequency, Some(165.0));
}

#[test]
fn test_mode_4k_resolution() {
    let result = Mode::from_line("3840x2160@60Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 3840);
    assert_eq!(mode.height, 2160);
}

#[test]
fn test_mode_ultrawide() {
    let result = Mode::from_line("3440x1440@100Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 3440);
    assert_eq!(mode.height, 1440);
}

#[test]
fn test_mode_fractional_refresh() {
    let result = Mode::from_line("1920x1080@59.94Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.frequency, Some(59.94));
}

// Negative tests for mode parsing

#[test]
fn test_mode_missing_x_separator() {
    let result = Mode::from_line("1920-1080@60Hz");
    assert!(result.is_err());
}

#[test]
fn test_mode_invalid_width() {
    let result = Mode::from_line("ABCDx1080@60Hz");
    assert!(result.is_err());
}

#[test]
fn test_mode_invalid_height() {
    let result = Mode::from_line("1920xABCD@60Hz");
    assert!(result.is_err());
}

#[test]
fn test_mode_invalid_refresh() {
    let result = Mode::from_line("1920x1080@ABCHz");
    assert!(result.is_err());
}

#[test]
fn test_mode_missing_hz_in_refresh() {
    let result = Mode::from_line("1920x1080@60H");
    assert!(result.is_err());
}

#[test]
fn test_mode_empty_string() {
    let result = Mode::from_line("");
    assert!(result.is_err());
}

#[test]
fn test_mode_only_width() {
    let result = Mode::from_line("1920");
    assert!(result.is_err());
}

#[test]
fn test_mode_negative_dimensions() {
    let result = Mode::from_line("-1920x1080");
    assert!(result.is_err());
}

#[test]
fn test_mode_zero_width() {
    let result = Mode::from_line("0x1080");
    // Parsing might succeed but validation should catch this
    // Adjust based on implementation
}

#[test]
fn test_mode_zero_height() {
    let result = Mode::from_line("1920x0");
    // Parsing might succeed but validation should catch this
    // Adjust based on implementation
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_mode_very_large_resolution() {
    let result = Mode::from_line("7680x4320@60Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 7680);
    assert_eq!(mode.height, 4320);
}

#[test]
fn test_mode_very_small_resolution() {
    let result = Mode::from_line("640x480@60Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 640);
    assert_eq!(mode.height, 480);
}

#[test]
fn test_mode_portrait_orientation() {
    let result = Mode::from_line("1080x1920@60Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 1080);
    assert_eq!(mode.height, 1920);
}

#[test]
fn test_mode_whitespace_trimming() {
    let result = Mode::from_line("  1920x1080@60Hz  ");
    assert!(result.is_ok());
}

#[test]
fn test_mode_custom_resolution() {
    let result = Mode::from_line("1234x567@89Hz");
    assert!(result.is_ok());
    let mode = result.unwrap();
    assert_eq!(mode.width, 1234);
    assert_eq!(mode.height, 567);
    assert_eq!(mode.frequency, Some(89.0));
}
