#!/usr/bin/env fish
# Setup virtual displays in headless Sway

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set PID_FILE $TEST_DIR/sway.pid

if not test -f $PID_FILE
    echo "❌ Sway not running. Start it first with: ./scripts/start-sway.fish"
    exit 1
end

set SWAY_PID (cat $PID_FILE)

# Find the actual Sway socket
set SWAY_SOCK (ls -t $XDG_RUNTIME_DIR/sway-ipc.*.$SWAY_PID.sock 2>/dev/null | head -1)

if test -z "$SWAY_SOCK"
    echo "❌ Could not find Sway socket for PID $SWAY_PID"
    exit 1
end

# Set SWAYSOCK for swaymsg commands
set -x SWAYSOCK $SWAY_SOCK

echo "Using Sway socket: $SWAY_SOCK"
echo "Creating virtual displays..."
echo ""

# Create two virtual outputs
swaymsg create_output
swaymsg create_output

echo "✓ Virtual displays created"
echo ""

# List outputs
echo "Current outputs:"
swaymsg -t get_outputs

# Get the Wayland display socket  
set WAYLAND_SOCK (swaymsg -t get_outputs | grep -o 'wayland-[0-9]*' | head -1)

echo ""
echo "You can now test with:"
echo "  export SWAYSOCK=$SWAY_SOCK"
if test -n "$WAYLAND_SOCK"
    echo "  export WAYLAND_DISPLAY=$WAYLAND_SOCK"
end
echo "  kanshi"
echo "  cargo run --bin kanske"
