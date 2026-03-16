# Headless Sway Testing Scripts

Scripts for testing kanske with a headless Sway compositor that simulates monitor hotplug events without physical displays.

## Quick Start

```fish
# Start headless Sway
./scripts/start-sway.fish

# In same or another terminal, source environment
source ./scripts/connect-sway.fish

# Test hotplug simulation
./scripts/simulate-hotplug.fish

# Run kanske
./scripts/test-kanske.fish

# When done
./scripts/stop-sway.fish
```

## Scripts

### Core Scripts

- **start-sway.fish** - Start headless Sway compositor
  - Creates isolated test environment in `$XDG_RUNTIME_DIR/kanske-test`
  - Saves socket paths for easy connection
  - Uses `WLR_BACKENDS=headless` for virtual displays
  - Returns socket paths to export

- **connect-sway.fish** - Source this to connect your shell to headless Sway
  - Exports `SWAYSOCK` and `WAYLAND_DISPLAY` environment variables
  - Must be sourced: `source ./scripts/connect-sway.fish`

- **stop-sway.fish** - Stop headless Sway and cleanup
  - Kills all related processes
  - Cleans up PID and socket files

### Testing Scripts

- **test-suite.fish** - Complete automated test workflow
  - Starts Sway, runs basic tests, demonstrates hotplug
  - Good for quick validation that everything works

- **simulate-hotplug.fish** - Demonstrate monitor hotplug simulation
  - Creates and destroys virtual outputs dynamically
  - Shows how to simulate real-world monitor events

- **test-kanske.fish** - Run kanske against headless Sway
  - Automatically sets up environment
  - Runs kanske daemon in foreground

### Development Scripts

- **debug-sway.fish** - Simple debug script to verify Sway startup
  - Minimal test case for troubleshooting
  - Useful if other scripts fail

## How It Works

1. **Headless Sway**: Uses wlroots headless backend (`WLR_BACKENDS=headless`)
   - Creates virtual outputs (HEADLESS-1, HEADLESS-2, etc.)
   - No GPU or physical displays needed
   - Software rendering with pixman

2. **IPC Socket**: Sway creates `sway-ipc.<uid>.<pid>.sock`
   - Used by `swaymsg` for control
   - Saved to `$XDG_RUNTIME_DIR/kanske-test/swaysock`

3. **Wayland Socket**: Sway creates `wayland-N` socket
   - Used by Wayland clients (like kanske)
   - Saved to `$XDG_RUNTIME_DIR/kanske-test/wayland_display`

## Environment Variables

When connected to headless Sway:
- `SWAYSOCK` - Path to Sway IPC socket (for swaymsg)
- `WAYLAND_DISPLAY` - Wayland display name (for kanske)

## Hotplug Simulation

```fish
# Connect to headless Sway first
source ./scripts/connect-sway.fish

# Create a new virtual display
swaymsg create_output

# List all outputs
swaymsg -t get_outputs

# Remove an output (simulates unplugging)
swaymsg output HEADLESS-2 unplug

# Configure output
swaymsg output HEADLESS-1 mode 1920x1080
swaymsg output HEADLESS-1 position 0,0
swaymsg output HEADLESS-1 scale 1.5
```

## Troubleshooting

### Sway won't start
```fish
# Check the log
cat $XDG_RUNTIME_DIR/kanske-test/sway.log

# Try the debug script
./scripts/debug-sway.fish
```

### Can't connect with swaymsg
```fish
# Verify environment is set
echo $SWAYSOCK
echo $WAYLAND_DISPLAY

# Re-source if needed
source ./scripts/connect-sway.fish
```

### Multiple Sway instances
```fish
# Clean up everything
./scripts/stop-sway.fish

# Kill any strays manually
pkill -9 -f "sway.*kanske-test"
```

## Architecture Notes

This setup follows the patterns used by:
- foot terminal's test suite
- swayvnc headless testing
- cagebreak development testing
- teams-for-linux Wayland testing

Key lessons learned:
- Don't use `exec` commands in Sway config (causes immediate exit)
- Must unset existing `SWAYSOCK` and `WAYLAND_DISPLAY` before starting
- Use `WLR_RENDERER=pixman` for reliable software rendering
- Socket path is predictable: `sway-ipc.<uid>.<pid>.sock`
- Can detect Wayland display from Sway IPC or by finding newest socket
