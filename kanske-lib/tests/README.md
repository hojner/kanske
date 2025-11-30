# Kanske Parser Tests

This directory contains comprehensive tests for the Kanske configuration file parser.

## Test Files

### `parser_tests.rs`
Integration tests that parse complete configuration files and verify the parser handles various scenarios correctly.

**Positive Tests (Valid Configurations):**
- Compact profiles with inline output settings
- Profiles with nested output blocks
- Mode parsing with various formats (with/without refresh rate, with/without Hz suffix)
- Custom mode flags
- All transform options (normal, 90, 180, 270, flipped variants)
- Enable/disable outputs
- Global output definitions with aliases
- Profiles using aliases
- Wildcard outputs
- Adaptive sync settings
- Exec commands
- Multi-monitor setups (triple monitor)
- Mixed quote styles (single, double, unquoted)
- HiDPI scaling
- Anonymous profiles
- Comment handling
- Empty line handling

**Negative Tests (Invalid Configurations):**
- Mismatched braces (open/close)
- Invalid mode formats (missing 'x', non-numeric values)
- Invalid refresh rates
- Invalid position formats
- Invalid scale values (non-numeric, zero, negative)
- Invalid transform values
- Invalid adaptive_sync values
- Missing output names
- Incomplete profile directives
- Unknown directives
- Aliases without dollar sign
- Multiple braces on same line
- Exec without command
- Unclosed quotes
- Nested profiles (not allowed)

### `unit_tests.rs`
Unit tests for individual parser components, focusing on the Mode parser.

**Mode Parsing Tests:**
- Valid mode formats with refresh rate and Hz suffix
- Valid mode formats with refresh rate without Hz suffix
- Valid mode formats without refresh rate
- High refresh rates (165Hz)
- 4K resolution
- Ultrawide resolutions
- Fractional refresh rates (59.94Hz)
- Custom resolutions
- Very large and very small resolutions
- Portrait orientation modes
- Whitespace trimming

**Negative Mode Tests:**
- Missing 'x' separator
- Invalid width/height (non-numeric)
- Invalid refresh rate
- Missing Hz suffix
- Empty strings
- Only width (no height)
- Negative dimensions
- Zero width/height

## Running Tests

Run all tests:
```bash
cargo test
```

Run only parser integration tests:
```bash
cargo test --test parser_tests
```

Run only unit tests:
```bash
cargo test --test unit_tests
```

Run a specific test:
```bash
cargo test test_mode_with_refresh_and_hz
```

Run tests with output:
```bash
cargo test -- --nocapture
```

## Test Configuration Examples

The tests are based on the examples in `kanshi-complete-example.config`, which demonstrates all supported directives:

1. **Global output defaults & aliases** - Define reusable output configurations
2. **Inline output settings** - All settings on one line
3. **Nested blocks** - Settings in { } blocks
4. **Mode variations** - 1920x1080, 1920x1080@60, 1920x1080@60Hz
5. **Transform options** - normal, rotations (90, 180, 270), flipped variants
6. **Position** - X,Y coordinates for multi-monitor layouts
7. **Scale** - Integer and fractional scaling (1.0, 1.5, 2.0)
8. **Adaptive sync** - VRR/FreeSync/G-Sync on/off
9. **Exec commands** - Shell commands to run when profile activates
10. **Wildcard outputs** - Match all other outputs with *

## Adding New Tests

When adding new parsing features:

1. Add positive test cases to `parser_tests.rs` showing valid usage
2. Add negative test cases showing error handling
3. Add unit tests to `unit_tests.rs` for new parsing components
4. Update this README with the new test categories

## Expected Behavior

Some tests may need adjustment based on the parser implementation:

- Empty profiles: may be valid or invalid
- Negative positions: may be valid for some use cases
- Duplicate directives: last one wins or error
- Conflicting enable/disable: last one wins or error

Update test assertions based on the actual desired behavior.
