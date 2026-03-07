# Coordinate-Based Targeting

For apps where UI Automation is unavailable (GPU-rendered, canvas, custom-drawn controls), use visual coordinate targeting via `capture` and `--cursor` overlay.

## When to use

- `snapshot` returns only `window` with no children
- Target elements are rendered by custom drawing (DirectX, OpenGL, HTML canvas, etc.)
- You need sub-element precision within a UIA control (e.g., specific cell in a custom grid)

## Core principle: image coordinates = window coordinates

`capture --hwnd N` captures the window client area at physical pixel resolution. The pixel coordinates in the saved image map directly to `--coord window` coordinates — no scaling, no offset needed:

```bash
desktopwright capture --hwnd 132456 --output screen.png
# If the target appears at pixel (400, 300) in screen.png:
desktopwright click --x 400 --y 300 --coord window --hwnd 132456
```

This holds across all DPI scaling factors and multi-monitor configurations.

> **Do not use `--max-width` / `--max-height` when capturing for coordinates.** Resized images have scaled coordinates that do not match `--coord window` values.

## Getting initial coordinates from `ui-tree`

`ui-tree` reports each element's bounding rectangle in window coordinates. For apps where the target is a UIA group or control (even if it has no children), `ui-tree` can give you the exact rect without any visual estimation:

```bash
desktopwright ui-tree --hwnd 132456
# → group "Canvas" [512, 200, 800×600]
# Center: x = 512 + 800/2 = 912, y = 200 + 600/2 = 500
desktopwright click --x 912 --y 500 --coord window --hwnd 132456
```

Use this as a faster alternative to iterative cursor overlay when the target element has a known UIA entry even if its content is not accessible.

## Iterative targeting with cursor overlay

When the initial coordinate estimate is off, use `move` + `capture --cursor` to converge on the exact position:

```bash
# Step 1: capture to get an initial estimate
desktopwright capture --hwnd 132456 --output screen.png

# Step 2: move cursor to estimated position and capture with overlay
desktopwright move --x 400 --y 300 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --cursor --output verify.png

# Step 3: examine verify.png — if cursor is on target, click; otherwise adjust
desktopwright click --x 400 --y 300 --coord window --hwnd 132456
```

### Reading the cursor overlay

The overlay drawn on `--cursor` images:

| Element | Description |
|---------|-------------|
| White dot (1px) | Exact cursor position |
| ±2px clear zone | No drawing — lets the background (target pixels) show through |
| Dashed arms | Extend to image edges, showing the cursor's row and column |
| Fibonacci tick marks | Perpendicular marks at 3, 5, 8, 13, 21, 34, 55, 89px from center |

**How to use tick marks for fine adjustment:**
- The clear zone allows you to see the target pixel at the cursor center
- If the target is visible but offset: count which tick mark the target falls near, move by that distance
- Example: target appears ~13px to the right of the cursor → add 13 to the x coordinate

**Convergence:** typical workflow reaches the target in 1–2 iterations. What counts as "aligned" depends on the target size and what the application requires — verify with a test click if unsure.

## Region capture for tight targets

When the target area is small and hard to estimate from the full window, capture a region first:

```bash
# Capture a 200×200 region starting at (300, 250) in the window
desktopwright capture --hwnd 132456 \
  --region-x 300 --region-y 250 \
  --region-width 200 --region-height 200 \
  --output region.png
```

Region pixel coordinates are relative to the region origin. To compute the window coordinate:

```
window_x = region-x + region_pixel_x
window_y = region-y + region_pixel_y
```

## Verifying the click landed correctly

After clicking, capture with `--wait-for-diff` to confirm the app responded:

```bash
desktopwright click --x 400 --y 300 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
# after.png contains the changed region if anything moved/updated
```

If no change occurs within the timeout:
- stderr: `タイムアウト: 3000ms 以内に変化がありませんでした`
- stdout: empty (no image saved)
- The click may have landed in the wrong place — repeat the cursor overlay check

## Drag operations

```bash
desktopwright drag \
  --from-x 100 --from-y 200 \
  --to-x 300 --to-y 400 \
  --coord window --hwnd 132456

# Slower drag with more intermediate steps (for apps that need smooth movement)
desktopwright drag \
  --from-x 100 --from-y 200 \
  --to-x 300 --to-y 400 \
  --steps 30 \
  --coord window --hwnd 132456
```

## Common patterns

### Click, verify response, adjust if needed

```bash
desktopwright capture --hwnd 132456 --output before.png

desktopwright move --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --cursor --output verify.png
# Inspect verify.png to confirm cursor is on target

desktopwright click --x 200 --y 150 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output after.png
# If after.png shows no change, cursor was off — re-examine verify.png and adjust
```

### Scroll then target

```bash
# Scroll to bring target into view, then re-capture for new coordinates
desktopwright scroll --direction down --amount 3 --x 400 --y 300
desktopwright capture --hwnd 132456 --output screen.png
# Re-identify target in the new screen.png
desktopwright move --x 400 --y 350 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --cursor --output verify.png
desktopwright click --x 400 --y 350 --coord window --hwnd 132456
```
