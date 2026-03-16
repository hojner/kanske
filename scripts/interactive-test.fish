#!/usr/bin/env fish
# Interactive testing session - run kanske and control it from the same shell

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

echo "=== Starting Interactive Test Session ==="
echo "Environment:"
echo "  SWAYSOCK=$SWAYSOCK"
echo "  WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
echo ""

# Start kanske in background
echo "Starting kanske in background..."
cargo run --bin kanske &
set KANSKE_PID $last_pid
echo "  kanske PID: $KANSKE_PID"
echo ""

# Give kanske a moment to initialize
sleep 2

echo "=== Interactive Commands ==="
echo "You can now run swaymsg commands to test hotplug:"
echo ""
echo "  swaymsg create_output          # Add a monitor"
echo "  swaymsg -t get_outputs         # List monitors"
echo "  swaymsg output HEADLESS-2 unplug  # Remove a monitor"
echo ""
echo "Kanske is running and will react to display changes."
echo ""
echo "When done:"
echo "  kill $KANSKE_PID               # Stop kanske"
echo "  ./scripts/stop-sway.fish       # Stop Sway"
echo ""
echo "Press Ctrl+C to stop kanske and exit this session"
echo ""

# Wait for kanske to exit (or user to Ctrl+C)
wait $KANSKE_PID
