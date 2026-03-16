#!/usr/bin/env fish
# Debug script to figure out why Sway doesn't create socket

echo "=== Testing Sway headless startup ==="
set TEST_DIR /tmp/sway-debug-test
mkdir -p $TEST_DIR

# Create config
echo "output * mode 1920x1080" > $TEST_DIR/sway.conf
echo "exec sleep infinity" >> $TEST_DIR/sway.conf

echo "Config:"
cat $TEST_DIR/sway.conf
echo ""

# Start Sway
echo "Starting Sway..."
env WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 sway -c $TEST_DIR/sway.conf >$TEST_DIR/sway.log 2>&1 &
set SWAY_PID $last_pid
echo "Started background process PID: $SWAY_PID"

# Wait and check
for i in (seq 1 10)
    sleep 1
    echo "Second $i:"
    
    # Find actual sway process
    set ACTUAL_PIDS (pgrep -f "sway -c $TEST_DIR/sway.conf")
    if test -n "$ACTUAL_PIDS"
        echo "  Found Sway processes: $ACTUAL_PIDS"
        for pid in $ACTUAL_PIDS
            set SOCK_PATH /run/user/1000/sway-ipc.1000.$pid.sock
            if test -S "$SOCK_PATH"
                echo "  ✓ Socket exists: $SOCK_PATH"
                echo ""
                echo "SUCCESS! Sway is running with IPC socket"
                echo "Kill with: kill $ACTUAL_PIDS"
                exit 0
            else
                echo "  ✗ No socket at: $SOCK_PATH"
            end
        end
    else
        echo "  No Sway process found"
    end
end

echo ""
echo "FAILED - Socket never created"
echo "Log output:"
cat $TEST_DIR/sway.log
echo ""
echo "Cleanup: kill "(pgrep -f "sway -c $TEST_DIR/sway.conf")" 2>/dev/null"
