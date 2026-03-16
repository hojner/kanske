#!/usr/bin/env fish
# Stop headless Sway compositor and ensure clean slate

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set PID_FILE $TEST_DIR/sway.pid
set SOCK_FILE $TEST_DIR/swaysock
set WAYLAND_FILE $TEST_DIR/wayland_display

echo "Cleaning up headless Sway instances..."

# Kill process from PID file if it exists
if test -f $PID_FILE
    set PID (cat $PID_FILE)
    
    if kill -0 $PID 2>/dev/null
        echo "Stopping Sway (PID: $PID)..."
        kill $PID
        sleep 1
        
        # Force kill if still running
        if kill -0 $PID 2>/dev/null
            echo "Force killing..."
            kill -9 $PID
        end
    end
end

# Kill any remaining headless Sway instances
set RUNNING_PIDS (pgrep -f "sway -c.*kanske-test/sway.conf" 2>/dev/null)
if test -n "$RUNNING_PIDS"
    echo "Killing remaining Sway processes: $RUNNING_PIDS"
    for pid in $RUNNING_PIDS
        kill -9 $pid 2>/dev/null
    end
    sleep 1
end

# Clean up test files
rm -f $PID_FILE $SOCK_FILE $WAYLAND_FILE

echo "✓ Clean slate ready"
