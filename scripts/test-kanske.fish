#!/usr/bin/env fish
# Test kanske with headless Sway

set SOCKET_NAME kanske-sway
set TEST_DIR $XDG_RUNTIME_DIR/kanske-test

# Check if Sway is running
if not test -S $XDG_RUNTIME_DIR/$SOCKET_NAME
    echo "❌ Sway socket not found at: $XDG_RUNTIME_DIR/$SOCKET_NAME"
    echo "Start Sway first with: ./scripts/start-sway.fish"
    exit 1
end

echo "=== Testing Sway Connection ==="
echo "Socket: $XDG_RUNTIME_DIR/$SOCKET_NAME"
echo ""

# Export the socket for kanske to use
set -x WAYLAND_DISPLAY $SOCKET_NAME

echo "Running kanske with headless Sway..."
echo ""
cargo run --bin kanske
