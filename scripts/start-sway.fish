#!/usr/bin/env fish
# Start Sway in headless mode for testing

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set PID_FILE $TEST_DIR/sway.pid
set LOG_FILE $TEST_DIR/sway.log

mkdir -p $TEST_DIR

# Check if already running
if test -f $PID_FILE
    set OLD_PID (cat $PID_FILE)
    if kill -0 $OLD_PID 2>/dev/null
        echo "⚠️  Sway already running (PID: $OLD_PID)"
        echo "Stop it first with: ./scripts/stop-sway.fish"
        exit 1
    else
        rm -f $PID_FILE
    end
end

echo "Starting headless Sway..."
echo "  Log: $LOG_FILE"
echo ""

# Start Sway in headless mode (it will create its own socket)
env WLR_BACKENDS=headless \
    WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c /dev/null > $LOG_FILE 2>&1 &

set SWAY_PID $last_pid
echo $SWAY_PID > $PID_FILE

# Wait for Sway to start
sleep 2

if kill -0 $SWAY_PID 2>/dev/null
    # Find the socket Sway created
    set SWAY_SOCK (ls -t $XDG_RUNTIME_DIR/sway-ipc.*.$SWAY_PID.sock 2>/dev/null | head -1)
    
    echo "✓ Sway started successfully (PID: $SWAY_PID)"
    echo "  Socket: $SWAY_SOCK"
    echo ""
    echo "Add virtual outputs with:"
    echo "  ./scripts/setup-displays.fish"
    echo ""
    echo "View logs:"
    echo "  tail -f $LOG_FILE"
    echo ""
    echo "Stop with:"
    echo "  ./scripts/stop-sway.fish"
else
    echo "❌ Sway failed to start. Check log:"
    echo "  cat $LOG_FILE"
    rm -f $PID_FILE
    exit 1
end
