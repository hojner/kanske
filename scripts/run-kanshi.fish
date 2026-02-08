#!/usr/bin/env fish
# Run kanshi connected to headless Sway for testing

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set PID_FILE $TEST_DIR/sway.pid

if not test -f $PID_FILE
    echo "❌ Sway not running. Start it first with: ./scripts/start-sway.fish"
    exit 1
end

set SWAY_PID (cat $PID_FILE)

# Find the Sway IPC socket
set SWAY_SOCK (ls -t $XDG_RUNTIME_DIR/sway-ipc.*.$SWAY_PID.sock 2>/dev/null | head -1)

if test -z "$SWAY_SOCK"
    echo "❌ Could not find Sway socket for PID $SWAY_PID"
    exit 1
end

# Get the newest Wayland socket (headless Sway creates it when it starts)
# We want the most recently created socket that isn't a lock file
set WAYLAND_SOCK (ls -t $XDG_RUNTIME_DIR/wayland-* 2>/dev/null | grep -v '\.lock$' | while read sock
    if test -S $sock  # Check if it's a socket
        basename $sock
    end
end | head -1)  # Get the NEWEST (head instead of tail)

echo "=== Headless Sway Test Environment ==="
echo "Sway PID: $SWAY_PID"
echo "Sway IPC Socket: $SWAY_SOCK"
echo "Wayland Socket: "(test -n "$WAYLAND_SOCK"; and echo $WAYLAND_SOCK; or echo "unknown")
echo ""
echo "Starting kanshi..."
echo "Press Ctrl+C to stop"
echo ""

# Run kanshi with isolated environment variables
# This doesn't affect your shell or real kanshi
env SWAYSOCK=$SWAY_SOCK WAYLAND_DISPLAY=$WAYLAND_SOCK kanshi
