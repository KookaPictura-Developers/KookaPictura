# M23 — CS6 UI chrome

Goal: make the shell look and feel like Photoshop CS6 without changing
behaviour. Reference: `docs/02-ui-ux/reference/cs6-workspace.png` (CS6
workspace, saved from a user screenshot). Corpus:
`docs/02-ui-ux/{application-frame,workspace-and-docks,toolbox-and-options-bar}.md`.
OpenSpec change `m23-cs6-ui-chrome` (MODIFIED `application-shell`,
`tool-framework`).

## Scope

- A CS6-style QSS stylesheet generated per brightness level and applied from
  `Theme::apply`: menu bar, options bar, panel dock tabs/title bars, tool
  buttons, status bar, scrollbars, menus, tooltips.
- The Tools panel rebuilt as a compact two-column icon grid with a
  foreground/background colour control and a screen-mode control.
- Default dock grouping: Color+Swatches, Layers+History,
  Navigator+Info+Histogram as tabbed docks.
- Dark document canvas and a styled document tab strip.

## Out of scope (later milestones)

- Pixel-exact CS6 metrics, icon art refresh, HUD/on-image displays.
- New panels (Gradients, Patterns, Properties, Adjustments, Libraries, Channels,
  Paths, Brush) and their contents.
- Workspace presets/switcher, icon-collapse docks, floating-panel drop zones.
- Layered panel internals (Layers lock row, blend/opacity row, bottom button
  strip) beyond what M20 already provides.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) stylesheet, (2) toolbox grid + colour control,
(3) dock grouping + canvas, (4) self-test.

## Verification

- `cmake -S . -B build && cmake --build build`
- fixture and no-argument self-tests exit 0 with new chrome checks (exit codes
  from 59)
- `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
