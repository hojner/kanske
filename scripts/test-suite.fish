#!/usr/bin/env fish
# Complete test workflow for kanske with headless Sway

echo "=== Kanske Headless Test Suite ==="
echo ""

# Start Sway
echo "1. Starting headless Sway..."
./scripts/start-sway.fish
if test $status -ne 0
    echo "❌ Failed to start Sway"
    exit 1
end
echo ""

# Load environment
source ./scripts/connect-sway.fish
echo ""

# Test basic connection
echo "2. Testing Sway connection..."
swaymsg -t get_version
echo ""

# Show initial outputs
echo "3. Initial outputs:"
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

# Simulate hotplug
echo "4. Simulating hotplug events..."
echo "   Adding monitor..."
swaymsg create_output >/dev/null
sleep 1
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

# Run kanske (if it exists)
if test -f target/debug/kanske
    echo "5. Running kanske..."
    cargo run --bin kanske &
    set KANSKE_PID $last_pid
    sleep 2
    kill $KANSKE_PID 2>/dev/null
    echo ""
else
    echo "5. Skipping kanske test (binary not built)"
    echo ""
end

echo "✓ Test suite complete"
echo ""
echo "Sway is still running. Use one of:"
echo "  ./scripts/simulate-hotplug.fish  - Test hotplug events"
echo "  ./scripts/test-kanske.fish       - Run kanske manually"
echo "  ./scripts/stop-sway.fish         - Stop Sway"
