#!/usr/bin/env fish
# Simple approach: just set up the environment and give you a shell

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set SOCK_FILE $TEST_DIR/swaysock
set WAYLAND_FILE $TEST_DIR/wayland_display

# Check if Sway is running
if not test -f $SOCK_FILE
    echo "❌ Sway not running. Start with: ./scripts/start-sway.fish"
    exit 1
end

set -gx SWAYSOCK (cat $SOCK_FILE)
set -gx WAYLAND_DISPLAY (cat $WAYLAND_FILE)

echo "=== Kanske Test Shell ==="
echo "Environment configured:"
echo "  SWAYSOCK=$SWAYSOCK"
echo "  WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
echo ""
echo "Common commands:"
echo "  cargo run --bin kanske &         # Run kanske in background"
echo "  kanshi &                         # Run kanshi in background"
echo "  jobs                             # List background jobs"
echo "  fg                               # Bring to foreground"
echo "  kill %1                          # Kill job 1"
echo ""
echo "  swaymsg create_output            # Add monitor"
echo "  swaymsg -t get_outputs           # List monitors"
echo "  swaymsg output HEADLESS-2 unplug # Remove monitor"
echo ""
echo "Type 'exit' when done"
echo ""

# Start an interactive shell with the environment preserved
exec fish
