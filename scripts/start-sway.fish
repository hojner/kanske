#!/usr/bin/env fish
# Start Sway in headless mode for testing

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set PID_FILE $TEST_DIR/sway.pid
set SOCK_FILE $TEST_DIR/swaysock
set WAYLAND_FILE $TEST_DIR/wayland_display
set LOG_FILE $TEST_DIR/sway.log
set CONFIG_FILE $TEST_DIR/sway.conf

mkdir -p $TEST_DIR

# Create minimal sway config
echo "# Minimal Sway config for headless testing" > $CONFIG_FILE
echo "output * mode 1920x1080" >> $CONFIG_FILE

# Check if already running
if test -f $PID_FILE
    set OLD_PID (cat $PID_FILE)
    if kill -0 $OLD_PID 2>/dev/null
        echo "⚠️  Sway already running (PID: $OLD_PID)"
        echo "Stop it first with: ./scripts/stop-sway.fish"
        exit 1
    else
        rm -f $PID_FILE $SOCK_FILE $WAYLAND_FILE
    end
end

echo "Starting headless Sway..."

# Start Sway in headless mode with necessary environment
env -u SWAYSOCK -u WAYLAND_DISPLAY \
    WLR_BACKENDS=headless \
    WLR_LIBINPUT_NO_DEVICES=1 \
    WLR_RENDERER=pixman \
    sway -c $CONFIG_FILE >$LOG_FILE 2>&1 &

set SWAY_PID $last_pid
echo $SWAY_PID >$PID_FILE

# Wait for Sway to start and create socket
echo "Waiting for Sway socket..."
set TIMEOUT 10
set ELAPSED 0
set SWAYSOCK ""

while test $ELAPSED -lt $TIMEOUT
    sleep 1
    set ELAPSED (math $ELAPSED + 1)

    # Check if socket exists using PID
    set POTENTIAL_SOCK $XDG_RUNTIME_DIR/sway-ipc.(id -u).$SWAY_PID.sock
    if test -S "$POTENTIAL_SOCK"
        set SWAYSOCK $POTENTIAL_SOCK
        break
    end
    
    # Check if process still running
    if not kill -0 $SWAY_PID 2>/dev/null
        echo "❌ Sway process died"
        cat $LOG_FILE
        exit 1
    end
end

if test -z "$SWAYSOCK"
    echo "❌ Sway socket not created after $TIMEOUT seconds"
    echo "Process status: "(ps -p $SWAY_PID -o state= 2>/dev/null || echo "dead")
    cat $LOG_FILE
    exit 1
end

# Verify Sway is responsive
if not env SWAYSOCK=$SWAYSOCK swaymsg -t get_version >/dev/null 2>&1
    echo "❌ Sway not responding to IPC"
    cat $LOG_FILE
    exit 1
end

# Get the Wayland display socket that Sway created
set WAYLAND_DISPLAY (env SWAYSOCK=$SWAYSOCK swaymsg -t get_outputs | grep -o 'wayland-[0-9]*' | head -1)
if test -z "$WAYLAND_DISPLAY"
    # Fallback: find most recent wayland socket
    set WAYLAND_DISPLAY (ls -t $XDG_RUNTIME_DIR/wayland-* 2>/dev/null | head -1 | xargs basename)
end

# Save socket paths for other scripts
echo $SWAYSOCK > $SOCK_FILE
echo $WAYLAND_DISPLAY > $WAYLAND_FILE

echo "✓ Sway started successfully (PID: $SWAY_PID)"
echo ""
echo "Export these to connect:"
echo "  set -gx SWAYSOCK $SWAYSOCK"
echo "  set -gx WAYLAND_DISPLAY $WAYLAND_DISPLAY"
echo ""
echo "Or source: source ./scripts/connect-sway.fish"
echo "Logs: tail -f $LOG_FILE"
