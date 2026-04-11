#!/usr/bin/env fish
# Test kanske with headless Sway

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set SOCK_FILE $TEST_DIR/swaysock
set WAYLAND_FILE $TEST_DIR/wayland_display

# Check if Sway is running
if not test -f $SOCK_FILE
    echo "❌ Sway not running. Start with: ./scripts/start-sway.fish"
    exit 1
end

set -x SWAYSOCK (cat $SOCK_FILE)
set -x WAYLAND_DISPLAY (cat $WAYLAND_FILE)

echo "=== Testing Sway Connection ==="
echo "SWAYSOCK: $SWAYSOCK"
echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
echo ""

# Verify Sway is responsive
echo "Testing swaymsg..."
swaymsg -t get_outputs
echo ""

echo "=== Running kanske ==="
RUST_LOG=trace cargo run --bin kanske
