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

function list_outputs
    swaymsg -t get_outputs | grep -oE '"name": "[^"]+"' | grep -oE '"[^"]+\"$' | tr -d '"'
end

echo "=== Simulating Monitor Hotplug Events ==="
echo "Initial outputs:"
set initial_outputs (list_outputs)
for o in $initial_outputs
    echo "  - $o"
end
echo ""

echo "Creating two monitors..."
swaymsg create_output
swaymsg create_output
sleep 1

set new_outputs (list_outputs)
set added_outputs
for o in $new_outputs
    if not contains $o $initial_outputs
        set -a added_outputs $o
    end
end

for o in $new_outputs
    echo "  - $o"
end
echo ""

for o in $added_outputs
    echo "Removing $o..."
    swaymsg output $o unplug
    sleep 1
end

echo "Final outputs:"
for o in (list_outputs)
    echo "  - $o"
end
echo ""

echo "✓ Hotplug simulation complete"
