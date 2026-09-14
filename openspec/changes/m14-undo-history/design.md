# M14 — Undo / Redo: Design

## Context

The app shell's mutating commands follow one shape: validate → mutate
`PictureViewRust::doc` (and sometimes `selection`) → `recomposite()`. There is
no state retention. `Document` and `Selection` are plain `Clone` value types,
so a snapshot history needs no engine cooperation, no inverse transforms, and
no command abstraction.

## Goals / Non-Goals

**Goals:**
- Bounded (20-state) undo/redo of every mutating app command.
- Restore the full editing state the commands mutate: document + selection.
- Testable without a Qt runtime, following the crate's helper-test pattern.
- App controls: Undo/Redo buttons + Ctrl+Z / Ctrl+Y shortcuts.

**Non-Goals:**
- Memory-optimized histories (per-layer COW, tile diffs, scratch files) — the
  working shell operates on small documents; full clones are the ceiling and
  the ponytail comment names the upgrade path.
- History palette UI with labeled states and random-access jumping (CS6's
  History panel) — only linear undo/redo this milestone.
- Undo across document close/open (loading a PSD resets history), and no
  persistence of history to disk.

## Decisions

- **Snapshots, not inverse ops.** Every alternative (command pattern with
  inverses, per-op diffs) multiplies code paths per command and gets
  adjustment-layer/filter interactions wrong. Full clones are trivially
  correct; the CS6 default depth is 20 states and we match it.
- **History is app state, not engine state.** `pictura-app/src/history.rs`
  owns a plain Rust struct `History` over `Snapshot { doc: Document,
  selection: Option<Selection> }`; `PictureViewRust` holds one. The engine
  crates stay history-agnostic (a script/automation consumer can adopt the
  module later by moving it; not speculative now).
- **Index-based stack, truncate-on-new-op.** `Vec<Snapshot>` plus a cursor:
  undo decrements, redo increments, a new successful op truncates the redo
  tail and appends. Two-stack alternatives buy nothing.
- **Capture inside each command, before mutation.** Each mutating command
  clones `(doc, selection)` after validation but before the engine call, and
  pushes only on success. Failed/invalid commands leave history untouched.
  Selection-only commands (`select_all`, `deselect`, `magic_wand`) capture
  too: they are history states in Photoshop and restoring them is free.
- **`open()` resets history.** A freshly loaded document has an empty stack;
  undoing across a file boundary would be surprising and wrong.
- **QObject surface:** `undo()`/`redo()` return `bool` (false when
  unavailable), `can_undo()`/`can_redo()` drive button enabled state, and the
  existing `changed()` signal carries the visual refresh. `history_depth()`
  exists for the self-test to assert stack behavior without poking internals.

## Risks / Trade-offs

- [Memory: 20 full-document clones] → Acceptable at working-shell scale;
  `ponytail:` comment marks COW/tile-diff as the upgrade path if PSB-size
  documents hit RAM.
- [Clone cost on every keystroke-scale command] → Commands are button-driven,
  not continuous; measure before optimizing.
- [Shortcut conflicts with future text-input widgets] → Shortcuts are
  `QShortcut` on the main window, not `QAction` on the application; revisit if
  dialogs gain text editing.

## Open Questions

None — the design is fully determined by the existing command shape.
