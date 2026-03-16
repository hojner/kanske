#!/usr/bin/env fish
# Source this to connect your shell to headless Sway

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set SOCK_FILE $TEST_DIR/swaysock
set WAYLAND_FILE $TEST_DIR/wayland_display

if not test -f $SOCK_FILE
    echo "❌ Sway not running. Start with: ./scripts/start-sway.fish"
    exit 1
end

set -gx SWAYSOCK (cat $SOCK_FILE)
set -gx WAYLAND_DISPLAY (cat $WAYLAND_FILE)

echo "✓ Connected to headless Sway"
echo "  SWAYSOCK=$SWAYSOCK"
echo "  WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
echo ""
echo "Test with: swaymsg -t get_outputs"
