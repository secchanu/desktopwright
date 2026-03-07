# UI Element Interaction

Interact with Windows apps through UI Automation (UIA) — the preferred approach when available.

## Snapshot anatomy

`snapshot` returns the accessible element tree in DFS order. Each element with a visible rect gets a `[ref=eN]` assigned:

```
# snapshot: "My App" (HWND: 132456)
- window "My App" [ref=e1]:
  - menubar [ref=e2]:
    - menuitem "File" [ref=e3]
    - menuitem "Edit" [ref=e4]
  - edit [ref=e5]:
    - value: "current text"
  - list [ref=e6]:
    - listitem "Item A" [ref=e7]
    - listitem "Item B" [ref=e8]
  - button "OK" [ref=e9]
  - button "Cancel" [ref=e10]
  - statusbar [ref=e11]:
    - text "Ready" [ref=e12]
```

- `ref` numbers are assigned fresh each time `snapshot` runs — do not reuse refs from a previous snapshot after the UI changes
- `value:` under an edit shows the current field content
- Elements with zero-size rects (hidden/collapsed) are omitted

## Clicking elements

```bash
# By ref — most reliable, no ambiguity
desktopwright click-element --hwnd 132456 --ref e9

# By text — matches the element name/label (partial match, case-insensitive)
desktopwright click-element --hwnd 132456 --text "OK" --role button

# By text only (role omitted — matches any role)
desktopwright click-element --hwnd 132456 --text "File"
```

If `--text` matches multiple elements, the command fails. Add `--role` to narrow down, or use `--ref`.

## Checkboxes and toggles

```bash
# Set checkbox to checked
desktopwright check --text "Enable feature" --hwnd 132456

# Set checkbox to unchecked
desktopwright uncheck --text "Enable feature" --hwnd 132456
```

These are idempotent — `check` on an already-checked box does nothing.

## Dropdowns and list boxes

```bash
# Select by value (the option text, exact match)
desktopwright select --value "Option A" --element "Mode" --hwnd 132456

# Select without --element when the control is unambiguous
desktopwright select --value "English" --hwnd 132456
```

## Dialogs

`dialog-accept` sends Enter (confirms the default button). `dialog-dismiss` sends Escape (cancels):

```bash
desktopwright dialog-accept
desktopwright dialog-dismiss
```

These work for simple yes/no prompts where Enter and Escape map to the intended action.

For dialogs with multiple named buttons (e.g., "Save" / "Don't Save" / "Cancel"), do not rely on Escape — instead snapshot the dialog, identify the button, and click it by ref:

```bash
# After the dialog appears, find its HWND
desktopwright list
desktopwright snapshot --hwnd <dialog_hwnd>
# → shows button "Don't Save" [ref=e4], button "Cancel" [ref=e5]
desktopwright click-element --hwnd <dialog_hwnd> --ref e4
```

## Reading element values

```bash
# Get current text of an edit field (returns the value)
desktopwright get-text --hwnd 132456 --role edit

# Get value of a specific named field
desktopwright get-text --hwnd 132456 --text "Username" --role edit

# Get text content of a label/status element
desktopwright get-text --hwnd 132456 --text "Status" --role text

# Wait up to 5s for element to appear
desktopwright get-text --hwnd 132456 --text "Result" --timeout 5000
```

Output is written to stdout. Use shell variable capture for assertions:

```bash
status=$(desktopwright get-text --hwnd 132456 --text "Status" --role text)
```

## Debugging the element tree

`ui-tree` shows the raw UIA tree including elements not visible in `snapshot`:

```bash
desktopwright ui-tree --hwnd 132456
```

Use `ui-tree` when:
- `snapshot` is missing elements you can see on screen
- You need the exact role/name strings for `--text`/`--role` arguments
- You want to understand the window's internal structure

## When UIA is limited

Some apps partially implement UIA:

**Electron apps** (VS Code, etc.) — UIA reaches the top-level window but not the web content inside. `snapshot` returns a shallow tree. Use coordinate-based targeting for content inside the web view.

**Custom-drawn controls** — Legacy Win32 apps may use owner-drawn controls that don't expose UIA names. `ui-tree` will show the control exists but with no name/value. Use coordinate targeting or keyboard shortcuts instead.

**Indication of UIA unavailability:**
```bash
desktopwright snapshot --hwnd 132456
# Output: "- window "My App" [ref=e1]:" with no children → UIA unavailable
```

See [coordinate-targeting.md](coordinate-targeting.md) for the fallback workflow.
