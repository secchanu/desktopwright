---
name: desktopwright
description: Automates Windows GUI application interactions. Use when the user needs to click buttons, type text, take screenshots, capture windows, interact with desktop apps, or perform E2E testing of Windows applications.
allowed-tools: Bash(desktopwright:*)
---

# Windows GUI Automation with desktopwright

## Quick start

```bash
# list open windows and find HWND
desktopwright list
# inspect UI elements — assigns [ref=eN] to each visible element
desktopwright snapshot --hwnd 132456
# interact using refs from the snapshot
desktopwright click-element --hwnd 132456 --ref e5
# type text
desktopwright type "Hello, World!"
# capture to verify the result
desktopwright capture --hwnd 132456 --output screen.png
# close the window
desktopwright close --hwnd 132456
```

## Commands

### Core

```bash
desktopwright list
desktopwright list --process notepad
desktopwright list --title "Chrome"
desktopwright list --all
desktopwright foreground
desktopwright snapshot --hwnd 132456
desktopwright ui-tree --hwnd 132456
desktopwright get-text --hwnd 132456 --text "Status" --role text
desktopwright get-text --hwnd 132456 --role edit
```

### Window

```bash
desktopwright capture --hwnd 132456 --output screen.png
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
desktopwright focus --hwnd 132456
desktopwright focus --target "メモ帳"
desktopwright window --hwnd 132456 --action minimize
desktopwright window --hwnd 132456 --action maximize
desktopwright window --hwnd 132456 --action restore
desktopwright resize --hwnd 132456 --width 1280 --height 720
```

### Mouse

```bash
desktopwright click --x 400 --y 200 --coord window --hwnd 132456
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --button right
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --double
desktopwright move --x 400 --y 300
desktopwright move --x 400 --y 300 --coord window --hwnd 132456
desktopwright drag --from-x 100 --from-y 200 --to-x 300 --to-y 400 --coord window --hwnd 132456
desktopwright mousedown --x 100 --y 200
desktopwright mouseup --x 100 --y 200
desktopwright scroll --direction down --amount 3
desktopwright scroll --direction up --amount 5
```

### Keyboard

```bash
desktopwright key enter
desktopwright key ctrl+c
desktopwright key ctrl+s
desktopwright press ctrl+shift+t
desktopwright keydown ctrl
desktopwright keyup ctrl
desktopwright type "Hello, World!"
```

### UI Elements

```bash
desktopwright click-element --hwnd 132456 --text "OK" --role button
desktopwright click-element --hwnd 132456 --ref e5
desktopwright check --text "同意する" --hwnd 132456
desktopwright uncheck --text "同意する" --hwnd 132456
desktopwright select --value "日本語" --element "言語" --hwnd 132456
desktopwright dialog-accept
desktopwright dialog-dismiss
```

### App lifecycle

```bash
desktopwright launch "C:\path\to\app.exe"
desktopwright launch "C:\path\to\app.exe" --delay 1000
desktopwright wait-for-window --process "app" --timeout 10000
desktopwright wait 500
desktopwright close --hwnd 132456
desktopwright close --target "無題 - メモ帳"
```

## Target specifiers

Windows are identified by HWND (most reliable), title partial match, or process name:

```bash
desktopwright focus --hwnd 132456
desktopwright focus --target "メモ帳"
desktopwright focus --process notepad
desktopwright focus --target "設定" --process SystemSettings
```

## Snapshots

After inspecting with `snapshot`, each visible element has a `[ref=eN]` identifier for use with `click-element --ref`:

```yaml
# snapshot: "メモ帳" (HWND: 132456)
- window "無題 - メモ帳" [ref=e1]:
  - menubar [ref=e2]:
    - menuitem "ファイル(F)" [ref=e3]
    - menuitem "編集(E)" [ref=e4]
  - edit [ref=e5]:
    - value: "テキスト内容"
  - statusbar [ref=e6]:
    - text "1行, 1列" [ref=e7]
```

Use `desktopwright click-element --ref e5 --hwnd 132456` to click by ref.

## --json flag

All commands support `--json` for machine-readable output:

```bash
desktopwright --json list
desktopwright --json capture --hwnd 132456 --output screen.png
desktopwright --json click-element --hwnd 132456 --text "OK" --role button
desktopwright --json wait-for-window --process "app" --timeout 10000
```

## Local installation

If `desktopwright` is not in PATH, use the full path or install to PATH:

```bash
.\desktopwright.exe list
.\desktopwright.exe install --skills
```

## Example: Basic UI interaction

```bash
desktopwright list
desktopwright snapshot --hwnd 132456
desktopwright click-element --hwnd 132456 --text "ファイル" --role menuitem
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output after.png
desktopwright close --hwnd 132456
```

## Example: E2E test flow

```bash
desktopwright launch "notepad.exe"
desktopwright wait-for-window --process "notepad" --timeout 5000
desktopwright list
desktopwright snapshot --hwnd 132456
desktopwright click-element --hwnd 132456 --ref e5
desktopwright type "テスト入力"
desktopwright key ctrl+s
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output result.png
desktopwright get-text --hwnd 132456 --role edit
desktopwright close --hwnd 132456
```

## Example: Coordinate-based interaction (UIA unavailable)

```bash
desktopwright list --process myapp
desktopwright capture --hwnd 132456 --output screen.png
desktopwright focus --hwnd 132456
desktopwright click --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output after.png
```

## Specific tasks

* **Getting started** [references/getting-started.md](references/getting-started.md)
* **Window targeting** [references/window-targeting.md](references/window-targeting.md)
* **Capture and diff detection** [references/capture.md](references/capture.md)
* **Mouse and keyboard input** [references/input.md](references/input.md)
* **Timing and waiting** [references/waiting.md](references/waiting.md)
* **E2E testing patterns** [references/e2e-testing.md](references/e2e-testing.md)
* **Troubleshooting** [references/troubleshooting.md](references/troubleshooting.md)
