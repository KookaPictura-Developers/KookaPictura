# M24 — Right panel: tab groups + icon rail

Goal: finish the CS6 right side. Three tabbed groups —
Color/Swatches/Gradients/Patterns, Properties/Adjustments/Libraries, and
Layers/Channels/Paths — plus a narrow right icon rail whose buttons toggle
collapsed panels (History, Actions, Info, Navigator, Histogram). Every panel is
also toggled from `Window > Panels`. Reference:
`docs/02-ui-ux/reference/cs6-workspace.png`. OpenSpec change `m24-panel-rail`
(new capability `panel-rail`; MODIFIED `application-shell`).

## Scope

- Eight placeholder panels (Gradients, Patterns, Properties, Adjustments,
  Libraries, Channels, Paths, Actions) with empty states.
- The three CS6 tab groups.
- The right icon rail and shared rail/`Window`-menu toggles.
- New `Window > Panels` commands for the eight panels.

## Out of scope (later milestones)

- Real content for the placeholder panels (gradient/pattern presets, Properties
  binding, adjustment presets, libraries, channel/path lists, actions).
- Icon-collapse auto-collapse, floating-panel drop zones, workspace presets and
  the workspace switcher, panel-title-bar menus.
- Rust, document-format, or dependency changes.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) placeholder panels + rail widget + CMake,
(2) frame grouping/rail/commands, (3) self-test.

## Verification

- `cmake -S . -B build && cmake --build build`
- fixture and no-argument self-tests exit 0 with new panel checks (exit codes
  from 62)
- `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
