## Context

The app is a single `crates/pictura-app/cpp/main.cpp` that builds a
`QMainWindow`, a custom `ImageView`, and one `QDockWidget` of debug buttons. The
Rust side is a cxx-qt `PictureView` QObject owning the document, selection,
history, and image. `CMakeLists.txt` lists `main.cpp` and `interop.cpp`
explicitly, and `--self-test` asserts engine behavior and exits non-zero on
failure (codes 1–24 today).

Constraints that shape this change:

- The shell is Qt Widgets, per `ARCH-003`; Rust supplies document state only.
- No new dependencies. The project already builds against system Qt 6 and wgpu.
- The project's test pattern for the shell is the in-binary `--self-test`, run
  under `xvfb`.
- The spec corpus (`docs/02-ui-ux`, `docs/11-cross-cutting`) is the long-form
  contract; OpenSpec carries per-change requirements.

## Goals / Non-Goals

**Goals:**

- A CS6-shaped frame: menu bar, central canvas, status bar, dock areas.
- One declarative command table driving the menu bar, with per-command
  enablement and a dispatch layer future menus/shortcuts/customization reuse.
- Full documented top-level menu tree, with unimplemented leaves disabled.
- Dark theme with four brightness levels; three screen modes; canvas-colour
  cycling.
- Panels registered as docks with layout persistence across restart.

**Non-Goals:**

- Multi-document tabs, New/Open/Save dialogs, and the file lifecycle (M17).
- Toolbox, tool state, and options bar contents (M18).
- Panels beyond the existing Layers dock; the Preferences dialog (M19).
- Custom menu/shortcut sets, a shortcut editor, a custom `QStyle`, and
  localization (later milestones).
- Any engine, PSD, or document-model behavior change.

## Decisions

### Command table lives in C++, not Rust

The registry is static C++ data in `commands.cpp`; Rust stays document-only.

- *Why:* menu construction is pure Qt; crossing structures over cxx-qt for a
  static table adds bridge surface for no behavior. Matches `ARCH-003`'s
  "widgets for the shell, Rust for data".
- *Alternatives considered:* a `pictura-ui` Rust crate (rejected: new crate,
  heavier bridge, no payoff until localization/custom sets land); string-matching
  action text (rejected in `UI-002`: ids must be stable across translations).
- *Revisit when:* `XC-001` localization or `UI-002` custom menu sets are built.

### Registry API: table + handler registration + dispatch by id

`CommandRegistry` owns the table (`id`, `path`, `label`, `shortcut`,
`implemented`), builds the menu bar with `QAction::setData(id)`, and dispatches
`triggered` to handlers registered by id. The frame registers handlers for the
commands it implements; the registry itself performs no action.

- *Why:* keeps the registry pure data/mechanism and the frame the only place that
  knows about screen modes and docks. Lets the menu tree be complete while
  handlers arrive incrementally.
- *Alternatives considered:* a `switch` on id inside the registry (rejected:
  couples registry to frame); per-action lambdas in the table (rejected: the
  table stops being serializable data, which future custom sets need).

### Enablement is a predicate re-evaluated on `aboutToShow`

Each `CommandSpec` carries a small `enabled()` predicate over available state
(`implemented` flag, `has_document()`, undo/redo availability). Menus recompute
on open; no rebuild.

- *Why:* the spec requires greyed-not-hidden and correct state on every open,
  without rebuilding menus on every document change.

### Extend `PictureView` with `has_document()` rather than a shell-side flag

The bridge already owns the document; the shell should not infer presence from a
possibly-generated fallback image.

- *Why:* today `open()` always yields a non-null image, so the frame cannot use
  the image to decide document-required enablement.

### Session persistence in C++ via `QSaveFile`; no `pictura-prefs` crate yet

Layout blob and brightness go to `$XDG_STATE_HOME/kooka-pictura/state.json`
(key/value text written atomically), loaded at startup.

- *Why:* the store is opaque UI state; a Rust TOML crate (`serde`/`toml`/
  `directories`, `XC-002`) is the eventual owner but is deferred. This avoids new
  dependencies and a second writer while the typed preference schema does not
  exist. Marked with a `ponytail:` comment naming the upgrade path.
- *Alternatives considered:* `QSettings` (rejected: `XC-002` forbids it as the
  source of truth); a Rust prefs crate now (deferred, not rejected).

### Theme via Fusion + `QPalette`, brightness as four palettes

A `Theme` unit applies Fusion and a generated `QPalette` per brightness level.
No custom `QStyle` yet.

- *Why:* `ARCH-003` fixes the mechanism (palette + style + sparing stylesheets);
  a `QProxyStyle` can replace the palette later without touching widgets.

### Freeze headers before parallel implementation

`theme.h`, `session.h`, `commands.h`, and `frame.h` are frozen (with the command
id list) before implementation slices are dispatched, so each slice owns a
disjoint file pair and integration is the orchestrator's job.

- *Why:* C++ compile coupling is unforgiving; disjoint file ownership and a fixed
  interface keep parallel work from colliding in `main.cpp`/`CMakeLists.txt`.

## Risks / Trade-offs

- **Full menu tree with mostly-disabled leaves looks inert** → It is the spec
  shape and makes enablement real; handlers fill in per milestone.
- **`saveState`/`restoreState` silently fails on duplicate/missing
  `objectName`s** → Enforce a unique `objectName` at dock registration and assert
  it in the self-test.
- **Wayland ignores absolute window placement; full-screen is
  compositor-controlled** → Restore relative dock layout only; the self-test runs
  offscreen/Xvfb, and screen modes degrade to no-ops where the compositor
  refuses.
- **Persistence format churn** → `state.json` carries a `schema_version`; unknown
  keys are preserved and a missing/corrupt file falls back to defaults.
- **Registry/frame interface drift during parallel work** → Interfaces frozen
  first; the orchestrator integrates and the self-test is the gate.
- **More `--self-test` exit codes** → Continue the existing monotonic numbering
  (new codes ≥ 25) and keep each assertion independent.

## Migration Plan

Additive. The existing debug dock remains as the `layers` panel; parameterized
operations (image/canvas resize, adjustments, filters) stay in that dock until
their dialog milestones. Rollback is reverting the `pictura-app` sources,
`CMakeLists.txt`, the bridge method, and the OpenSpec artifacts; no document or
engine data changes.

## Open Questions

- Exact CS6 theme values (four brightness levels, panel/tab metrics) are
  unspecified in the corpus. M16 uses a defensible dark ramp; a later
  screenshot-diff pass fixes it.
- Whether the view-options triangle needs its full ten-item list in M16 or the
  document-size entry only. Plan: implement the popup with the documented list,
  wire the two entries the shell can compute.
- `Space+F` canvas-colour cycling under Wayland is unverified; treat the
  background as a shell-level value independent of compositor behaviour.
