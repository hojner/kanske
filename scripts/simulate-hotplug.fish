#!/usr/bin/env fish
# Simulate monitor hotplug events with headless Sway

set TEST_DIR $XDG_RUNTIME_DIR/kanske-test
set SOCK_FILE $TEST_DIR/swaysock

# Check if Sway is running
if not test -f $SOCK_FILE
    echo "❌ Sway not running. Start with: ./scripts/start-sway.fish"
    exit 1
end

set -x SWAYSOCK (cat $SOCK_FILE)

echo "=== Simulating Monitor Hotplug Events ==="
echo "Initial outputs:"
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

echo "Creating second monitor (HEADLESS-2)..."
swaymsg create_output HEADLESS-2
sleep 1
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

echo "Creating third monitor (HEADLESS-3)..."
swaymsg create_output HEADLESS-3
sleep 1
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

echo "Removing second monitor..."
swaymsg output HEADLESS-2 unplug
sleep 1
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

echo "Removing third monitor..."
swaymsg output HEADLESS-3 unplug
sleep 1
swaymsg -t get_outputs | grep -E '"name"' | sed 's/.*"name": "\([^"]*\)".*/  - \1/'
echo ""

echo "✓ Hotplug simulation complete"
