# desktopwright

GUI automation CLI tool for AI agents and E2E testing. Inspect UI elements, click buttons, type text, capture screenshots, and orchestrate desktop applications — all from the command line.

> **Platform support**: Windows is currently implemented. Linux and macOS support is planned.

## Features

- **Accessibility-first**: inspect UI trees via the platform's accessibility API and interact by element name or `[ref=eN]` — no fragile coordinates required
- **Screenshot capture**: capture any window and detect visual changes with `--wait-for-diff`
- **Full input control**: mouse, keyboard, drag, scroll
- **JSON output**: every command supports `--json` for machine-readable results
- **AI agent ready**: install a [Claude Code](https://github.com/anthropics/claude-code) skill with a single command so any Claude session can drive it immediately

## Requirements

- Windows 10 or later (current), Linux and macOS (planned)
- No admin rights required for most operations

## Installation

Download the latest binary from [Releases](https://github.com/secchanu/desktopwright/releases) and place it somewhere on your `PATH`.

```bash
desktopwright --version
```

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

| Category | Commands |
|---|---|
| Core | `list`, `foreground`, `snapshot`, `ui-tree`, `get-text` |
| Window | `capture`, `focus`, `window`, `resize`, `close` |
| Mouse | `click`, `move`, `drag`, `mousedown`, `mouseup`, `scroll` |
| Keyboard | `key`, `keydown`, `keyup`, `type`, `press` |
| UI Elements | `click-element`, `check`, `uncheck`, `select`, `dialog-accept`, `dialog-dismiss` |
| App lifecycle | `launch`, `wait-for-window`, `wait` |

See `desktopwright --help` or the [skill documentation](skills/desktopwright/SKILL.md) for full usage.

## Snapshots

`snapshot` outputs an accessibility tree in YAML, assigning a `[ref=eN]` identifier to each visible element:

```yaml
# snapshot: "メモ帳" (HWND: 132456)
- window "無題 - メモ帳" [ref=e1]:
  - menubar [ref=e2]:
    - menuitem "ファイル(F)" [ref=e3]
    - menuitem "編集(E)" [ref=e4]
  - edit [ref=e5]
  - statusbar [ref=e6]:
    - text "1行, 1列" [ref=e7]
```

Use `click-element --ref eN` to interact by ref without specifying coordinates.

## JSON output

All commands support `--json` for structured output:

```bash
desktopwright --json list
desktopwright --json capture --hwnd 132456 --output screen.png
desktopwright --json click-element --hwnd 132456 --text "OK" --role button
desktopwright --json wait-for-window --process "app" --timeout 10000
```

## AI agent integration

Install a [Claude Code](https://github.com/anthropics/claude-code) skill so any Claude session can use desktopwright without additional setup:

```bash
# install to current project
desktopwright install --skills

# install globally (~/.claude/skills/desktopwright)
desktopwright install --skills --global
```

Once installed, Claude will automatically load the skill and can execute desktopwright commands directly.

## Build from source

```bash
git clone https://github.com/secchanu/desktopwright
cd desktopwright
cargo build --release
```

The binary is output to `target/release/desktopwright` (`desktopwright.exe` on Windows).

## Acknowledgements

- [playwright-cli](https://github.com/microsoft/playwright-cli) — inspired the CLI design, skill format, and AI agent integration approach
