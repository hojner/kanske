#!/usr/bin/env fish
# Stop headless Sway compositor

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set PID_FILE $TEST_DIR/sway.pid

if not test -f $PID_FILE
    echo "⚠️  No Sway PID file found"
    exit 0
end

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
    
    echo "✓ Sway stopped"
else
    echo "⚠️  Sway not running"
end

rm -f $PID_FILE
