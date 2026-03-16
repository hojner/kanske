#!/usr/bin/env fish
# Run kanske and kanshi side-by-side in tmux for comparison testing

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

# Check if tmux is available
if not command -v tmux >/dev/null
    echo "❌ tmux not found. Install with: sudo apt install tmux"
    exit 1
end

echo "=== Starting tmux session with kanske ==="
echo ""

# Create a tmux session with split panes
tmux new-session -d -s kanske-test "SWAYSOCK=$SWAYSOCK WAYLAND_DISPLAY=$WAYLAND_DISPLAY cargo run --bin kanske"
tmux split-window -h -t kanske-test "SWAYSOCK=$SWAYSOCK WAYLAND_DISPLAY=$WAYLAND_DISPLAY fish"
tmux send-keys -t kanske-test:0.1 "echo '=== Control Panel ==='" C-m
tmux send-keys -t kanske-test:0.1 "echo 'Kanske is running in left pane'" C-m
tmux send-keys -t kanske-test:0.1 "echo ''" C-m
tmux send-keys -t kanske-test:0.1 "echo 'Test hotplug with:'" C-m
tmux send-keys -t kanske-test:0.1 "echo '  swaymsg create_output'" C-m
tmux send-keys -t kanske-test:0.1 "echo '  swaymsg -t get_outputs'" C-m
tmux send-keys -t kanske-test:0.1 "echo '  swaymsg output HEADLESS-2 unplug'" C-m
tmux send-keys -t kanske-test:0.1 "echo ''" C-m
tmux send-keys -t kanske-test:0.1 "echo 'Exit: Ctrl+D or type exit'" C-m
tmux send-keys -t kanske-test:0.1 "echo ''" C-m

# Attach to the session
tmux attach-session -t kanske-test
