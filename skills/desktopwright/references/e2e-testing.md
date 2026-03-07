# E2E Testing Patterns

Patterns for automating and verifying Windows desktop application behavior.

## Basic test structure

```bash
# 1. Launch
desktopwright launch "C:\path\to\app.exe"
desktopwright wait-for-window --process "myapp" --timeout 10000
desktopwright --json list --process "myapp"

# 2. Interact
desktopwright snapshot --hwnd 132456
desktopwright click-element --hwnd 132456 --ref e5
desktopwright type "test input"

# 3. Verify
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output result.png
result=$(desktopwright get-text --hwnd 132456 --text "Status" --role text)

# 4. Teardown
desktopwright close --hwnd 132456
```

## UIA-based test

For apps with accessible UI elements:

```bash
desktopwright launch "C:\path\to\app.exe"
# --json returns { "hwnd": 132456, ... } immediately when the window appears
hwnd=$(desktopwright --json wait-for-window --process "myapp" --timeout 5000 | grep -o '"hwnd":[0-9]*' | grep -o '[0-9]*')

# Fill a form
desktopwright snapshot --hwnd $hwnd
desktopwright click-element --hwnd $hwnd --text "Name" --role edit
desktopwright type "Test User"
desktopwright click-element --hwnd $hwnd --text "Submit" --role button

# Wait for confirmation and assert
desktopwright capture --hwnd $hwnd --wait-for-diff 5000 --output confirm.png
status=$(desktopwright get-text --hwnd $hwnd --text "Result" --role text)

desktopwright close --hwnd $hwnd
```

## Coordinate-based test (UIA unavailable)

For GPU-rendered or custom-drawn apps:

```bash
desktopwright launch "C:\path\to\app.exe"
desktopwright wait-for-window --process "myapp" --timeout 10000

# Verify UIA is unavailable
desktopwright snapshot --hwnd 132456
# → "- window "My App" [ref=e1]:" with no children

# Capture and identify button location visually
desktopwright capture --hwnd 132456 --output screen.png

# Verify cursor before clicking
desktopwright move --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --cursor --output verify.png
# Inspect verify.png: white dot should be on the button

desktopwright click --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png

desktopwright close --hwnd 132456
```

## Assertions

### Text value assertion

```bash
actual=$(desktopwright get-text --hwnd 132456 --text "Count" --role text)
expected="42"
if [ "$actual" = "$expected" ]; then
    echo "PASS"
else
    echo "FAIL: expected=$expected actual=$actual"
    exit 1
fi
```

### Visual change assertion

```bash
desktopwright click-element --hwnd 132456 --ref e5
# Capture with wait-for-diff: empty stdout = no change occurred (fail)
output=$(desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png)
if [ -z "$output" ]; then
    echo "FAIL: no UI change after action"
    exit 1
fi
```

### JSON-based assertion

```bash
result=$(desktopwright --json click-element --hwnd 132456 --text "OK" --role button)
# result contains: { "name": "OK", "role": "button", "rect": {...}, "click_x": 400, "click_y": 200 }
```

## Dialog handling

```bash
# Trigger an action that may show a dialog
desktopwright click-element --hwnd 132456 --text "Delete" --role button

# Wait for dialog window to appear
desktopwright wait-for-window --target "Confirm" --timeout 3000

# Accept or dismiss
desktopwright dialog-accept
# or
desktopwright dialog-dismiss

# Verify dialog is gone and app state changed
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output after-dialog.png
```

## Save dialog / file picker

```bash
# Trigger save (e.g., Ctrl+S opens a file save dialog)
desktopwright focus --hwnd 132456
desktopwright key ctrl+s

# Wait for file dialog window
desktopwright wait-for-window --target "Save" --timeout 3000
desktopwright list  # find dialog HWND

# Interact with dialog via UIA (standard file dialogs support UIA)
desktopwright snapshot --hwnd 99999
desktopwright click-element --hwnd 99999 --text "File name" --role edit
desktopwright type "output.txt"
desktopwright click-element --hwnd 99999 --text "Save" --role button
```

## Best practices

### Always verify after interaction

Never assume an operation succeeded. Capture after every significant action:

```bash
desktopwright click-element --hwnd 132456 --ref e9
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
# Inspect after.png before proceeding
```

### Use `--json list` to get HWND programmatically

```bash
desktopwright --json list --process "myapp"
# → [{ "hwnd": 132456, "title": "My App", "pid": 5678, ... }]
```

### Prefer `wait-for-window` over fixed delays after launch

```bash
# Preferred
desktopwright launch "C:\path\to\app.exe"
desktopwright wait-for-window --process "myapp" --timeout 10000

# Avoid
desktopwright launch "C:\path\to\app.exe" --delay 3000  # may be too short or too long
```

### UAC-elevated apps require elevated desktopwright

If `focus` fails or clicks have no effect on an admin-mode app, run desktopwright from an elevated shell (Run as Administrator).
