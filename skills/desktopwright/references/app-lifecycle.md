# App Lifecycle & Timing

Managing app launch, window state, and timing between operations.

## Launching apps

```bash
# Launch and continue immediately
desktopwright launch "C:\path\to\app.exe"

# Launch with arguments
desktopwright launch "C:\path\to\app.exe" "arg1" "arg2"

# Launch and wait before the next command (fixed delay)
desktopwright launch "C:\path\to\app.exe" --delay 1000
```

After `launch`, the window may not appear immediately. Use `wait-for-window` to wait reliably instead of a fixed delay.

## Waiting for a window to appear

```bash
# Wait up to 10s for any window from the process
desktopwright wait-for-window --process "myapp" --timeout 10000

# Wait for a window with a specific title
desktopwright wait-for-window --target "My App" --timeout 5000

# Get the HWND in JSON for use in subsequent commands
desktopwright --json wait-for-window --process "myapp" --timeout 10000
# → { "hwnd": 132456, "title": "My App", ... }
```

Exits with error if no matching window appears within the timeout.

> **UWP / Windows Store apps** (Calculator, Settings, etc.) are hosted under the `ApplicationFrameHost` process, not under their own process name. `--process "CalculatorApp"` will time out. Use `--target` with a title keyword, or use `--json list --all` and filter by title instead:
>
> ```bash
> desktopwright --json list --all
> # Find the window by title (e.g., "電卓") then use its hwnd
> ```

## Window state

```bash
# Restore a minimized window before capturing or interacting
desktopwright window --hwnd 132456 --action restore

desktopwright window --hwnd 132456 --action minimize
desktopwright window --hwnd 132456 --action maximize

# Set window size (e.g., for consistent captures)
desktopwright resize --hwnd 132456 --width 1280 --height 720
```

Minimized windows cannot be captured. Always `restore` first if needed.

## Keyboard focus

`key` and `type` send input to the foreground window. Always `focus` before keyboard operations:

```bash
desktopwright focus --hwnd 132456
desktopwright type "input text"
desktopwright key ctrl+s
```

Run `focus` immediately before the keyboard command — other operations between them can steal focus.

To confirm the correct window has focus before typing:
```bash
desktopwright foreground
```

## Waiting for UI changes

### `--wait-for-diff` (preferred)

Waits up to N milliseconds for any visible change in the window. Returns immediately when a change is detected:

```bash
desktopwright click-element --hwnd 132456 --ref e5
desktopwright capture --hwnd 132456 --wait-for-diff 5000 --output after.png
```

- No change detected within timeout: exits normally, stdout empty, stderr reports timeout
- Use `--json` to get the changed region bounding box: `{ "changed_region": { "x": ..., "y": ..., "width": ..., "height": ... } }`

Increase `--diff-threshold` (default 0.05) to ignore minor noise like cursor blinking:

```bash
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --diff-threshold 0.1 --output after.png
```

### `--delay`

Fixed wait before executing the command. Use when the change is known to be gradual (animations, tooltips):

```bash
# Wait 500ms after the command is issued
desktopwright capture --hwnd 132456 --delay 500 --output after.png
desktopwright key alt+f --delay 300
```

### `wait`

Standalone fixed wait. Rarely needed — prefer `--wait-for-diff`:

```bash
desktopwright wait 500
```

## Closing apps

```bash
desktopwright close --hwnd 132456
desktopwright close --target "My App"
desktopwright close --process "myapp"
```

`close` sends `WM_CLOSE`, which gives the app a chance to handle unsaved changes (a save dialog may appear). Capture after `close` to check if a dialog opened.

If a dialog appears, do not rely on `dialog-dismiss` alone — it sends Escape, which may cancel the dialog rather than clicking the intended button (e.g., "Don't Save"). Instead, find the dialog window, snapshot it, identify the button by ref, and click it:

```bash
desktopwright close --hwnd 132456
# Check if a dialog appeared
desktopwright capture --hwnd 132456 --wait-for-diff 1000 --output after-close.png
# If a dialog is visible, find it with list and snapshot it
desktopwright list
desktopwright snapshot --hwnd <dialog_hwnd>
# → shows button "Don't Save" [ref=e4], button "Cancel" [ref=e5]
desktopwright click-element --hwnd <dialog_hwnd> --text "Don't Save" --role button
```

## Common patterns

### Launch → interact → close

```bash
desktopwright launch "C:\path\to\app.exe"
desktopwright wait-for-window --process "myapp" --timeout 10000
desktopwright --json list --process "myapp"
# use the returned hwnd in all subsequent commands
desktopwright snapshot --hwnd 132456
# ... interact ...
desktopwright close --hwnd 132456
# check for unsaved-changes dialog
desktopwright capture --hwnd 132456 --wait-for-diff 1000 --output after-close.png
```

### Action → wait for response

```bash
# Click triggers async operation
desktopwright click-element --hwnd 132456 --ref e9
# Wait for the response to appear (up to 10s)
desktopwright capture --hwnd 132456 --wait-for-diff 10000 --output result.png
```

### Menu open → animation settle → capture

```bash
desktopwright click-element --hwnd 132456 --ref e3
# Wait for menu animation to finish before reading the contents
desktopwright capture --hwnd 132456 --delay 300 --output menu.png
```
