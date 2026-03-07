---
name: desktopwright
description: Automates desktop GUI interactions for AI agents and E2E testing. Use when the user needs to click, type, take screenshots, or test desktop applications.
allowed-tools: Bash(desktopwright:*)
---

# Windows GUI Automation with desktopwright

## Quick start

```bash
# 1. Find the target window and get its HWND
desktopwright list
# 2. Inspect UI elements — each visible element gets a [ref=eN]
desktopwright snapshot --hwnd 132456
# 3. Interact using refs from the snapshot
desktopwright click-element --hwnd 132456 --ref e5
desktopwright type "Hello, World!"
# 4. Always capture to verify the result
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
```

## Commands

### Window

```bash
desktopwright list
desktopwright list --process notepad
desktopwright list --title "My App"
desktopwright list --all
desktopwright foreground
desktopwright focus --hwnd 132456
desktopwright focus --target "My App"
desktopwright focus --process myapp
desktopwright window --hwnd 132456 --action minimize
desktopwright window --hwnd 132456 --action maximize
desktopwright window --hwnd 132456 --action restore
desktopwright resize --hwnd 132456 --width 1280 --height 720
```

### Capture

```bash
desktopwright capture --hwnd 132456 --output screen.png
desktopwright capture --hwnd 132456 --cursor --output verify.png
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
desktopwright capture --hwnd 132456 --region-x 0 --region-y 0 --region-width 400 --region-height 300 --output crop.png
```

### UI Elements (UIA)

```bash
desktopwright snapshot --hwnd 132456
desktopwright ui-tree --hwnd 132456
desktopwright click-element --hwnd 132456 --ref e5
desktopwright click-element --hwnd 132456 --text "OK" --role button
desktopwright get-text --hwnd 132456 --text "Status" --role text
desktopwright get-text --hwnd 132456 --role edit
desktopwright check --text "Agree" --hwnd 132456
desktopwright uncheck --text "Agree" --hwnd 132456
desktopwright select --value "Option A" --element "Dropdown" --hwnd 132456
desktopwright dialog-accept
desktopwright dialog-dismiss
```

### Mouse

```bash
desktopwright click --x 400 --y 200 --coord window --hwnd 132456
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --button right
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --double
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --direct
desktopwright move --x 400 --y 300 --coord window --hwnd 132456
desktopwright drag --from-x 100 --from-y 200 --to-x 300 --to-y 400 --coord window --hwnd 132456
desktopwright mousedown --x 100 --y 200 --coord window --hwnd 132456
desktopwright mouseup --x 100 --y 200 --coord window --hwnd 132456
desktopwright scroll --direction down --amount 3
desktopwright scroll --direction up --x 500 --y 400 --amount 5
```

### Keyboard

```bash
desktopwright focus --hwnd 132456
desktopwright type "Hello, World!"
desktopwright key enter
desktopwright key ctrl+c
desktopwright key ctrl+s
desktopwright keydown ctrl
desktopwright keyup ctrl
```

### App lifecycle

```bash
desktopwright launch "notepad.exe"
desktopwright launch "C:\path\to\app.exe" "arg1"
desktopwright wait-for-window --process "notepad" --timeout 10000
desktopwright wait-for-window --target "My App" --timeout 5000
desktopwright wait 500
desktopwright close --hwnd 132456
desktopwright close --target "My App"
desktopwright close --process "myapp"
```

## Finding windows

`list` shows all visible windows with their HWND, title, and process name:

```bash
desktopwright list
desktopwright --json list
# → [{ "hwnd": 132456, "title": "My App", "pid": 5678, "process": "myapp" }]
```

Use `--all` to include windows hidden from the default list (empty-title windows, background processes):

```bash
desktopwright list --all
desktopwright list --all --process "ApplicationFrameHost"
```

> **UWP / Windows Store apps** (Calculator, Settings, etc.) run under the `ApplicationFrameHost` process. `--process "CalculatorApp"` returns nothing. Use `--target` with a title keyword or `--all` to find them:
>
> ```bash
> desktopwright list --all
> desktopwright wait-for-window --target "Calculator" --timeout 5000
> ```

After `launch`, always wait before issuing commands:

```bash
desktopwright launch "notepad.exe"
desktopwright wait-for-window --process "notepad" --timeout 10000
desktopwright --json list --process "notepad"
```

## Snapshot and capture

**desktopwright does not provide automatic feedback after commands.** Always run `snapshot` or `capture` to see the current state.

`snapshot` shows the UI element tree:

```
# snapshot: "My App" (HWND: 132456)
- window "My App" [ref=e1]:
  - menubar [ref=e2]:
    - menuitem "File" [ref=e3]
    - menuitem "Edit" [ref=e4]
  - edit [ref=e5]:
    - value: "current text"
  - button "OK" [ref=e6]
  - statusbar [ref=e7]:
    - text "Ready" [ref=e8]
```

`capture` saves a screenshot. Use `--wait-for-diff` after every interaction to confirm the UI responded:

```bash
desktopwright click-element --hwnd 132456 --ref e6
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
```

If `--wait-for-diff` times out (stdout empty, no image saved), the action likely missed its target — re-examine coordinates or the snapshot.

## UIA or coordinates?

Run `snapshot` first. The output determines which approach to use.

**UIA available** — snapshot shows named child elements:

```
- window "My App" [ref=e1]:
  - button "OK" [ref=e2]
  - edit [ref=e3]:
    - value: "hello"
```

→ Use `click-element --ref eN` or `click-element --text "name" --role role`.

**UIA unavailable** — snapshot returns only the top-level window:

```
- window "My App" [ref=e1]:
```

→ Use coordinate-based targeting. See [references/coordinate-targeting.md](references/coordinate-targeting.md).

**Partial UIA** (canvas apps, Electron) — snapshot shows a container with no accessible content inside. Use `ui-tree` to get bounding rectangles, then switch to coordinates for the content within.

## Keyboard input

`key` and `type` send input to the foreground window. Always `focus` immediately before:

```bash
desktopwright focus --hwnd 132456
desktopwright type "Hello"
desktopwright key ctrl+s
```

Other operations between `focus` and the keyboard command can steal focus — keep them adjacent.

## Dialogs

`dialog-accept` sends Enter (confirms the default button). `dialog-dismiss` sends Escape (cancels).

For dialogs with multiple named buttons (e.g., "Save" / "Don't Save" / "Cancel"), use `snapshot` on the dialog and `click-element` by ref instead:

```bash
desktopwright close --hwnd 132456
desktopwright list
desktopwright snapshot --hwnd <dialog_hwnd>
desktopwright click-element --hwnd <dialog_hwnd> --text "Don't Save" --role button
```

## --json flag

All commands support `--json` for machine-readable output:

```bash
desktopwright --json list
desktopwright --json list --process "notepad"
desktopwright --json capture --hwnd 132456 --output screen.png
desktopwright --json wait-for-window --target "My App" --timeout 10000
desktopwright --json click-element --hwnd 132456 --text "OK" --role button
```

## Example: form input

```bash
desktopwright list
desktopwright snapshot --hwnd 132456
# Snapshot shows: e1 edit "Username", e2 edit "Password", e3 button "Sign In"
desktopwright click-element --hwnd 132456 --ref e1
desktopwright type "user@example.com"
desktopwright click-element --hwnd 132456 --ref e2
desktopwright type "password"
desktopwright click-element --hwnd 132456 --ref e3
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output result.png
```

## Example: coordinate-based interaction

```bash
desktopwright list --all
desktopwright snapshot --hwnd 132456
# Snapshot shows only: - window "My App" [ref=e1]:  → UIA unavailable
desktopwright capture --hwnd 132456 --output screen.png
# Identify target in screen.png, estimate coordinates (e.g., x=200, y=150)
desktopwright move --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --cursor --output verify.png
# Confirm white dot is on target in verify.png, then click
desktopwright click --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
```

## Example: launch and close

```bash
desktopwright launch "notepad.exe"
desktopwright wait-for-window --process "notepad" --timeout 10000
hwnd=$(desktopwright --json list --process "notepad" | grep -o '"hwnd":[0-9]*' | grep -o '[0-9]*' | head -1)
desktopwright snapshot --hwnd $hwnd
# ... interact ...
desktopwright close --hwnd $hwnd
```

## Specific tasks

* **UI element interaction** [references/element-interaction.md](references/element-interaction.md)
* **Coordinate-based targeting** [references/coordinate-targeting.md](references/coordinate-targeting.md)
* **App lifecycle & timing** [references/app-lifecycle.md](references/app-lifecycle.md)
* **E2E testing patterns** [references/e2e-testing.md](references/e2e-testing.md)
