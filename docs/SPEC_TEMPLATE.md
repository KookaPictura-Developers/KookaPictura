# SPEC TEMPLATE

Copy this template for every feature spec. Keep the headings and order identical
so specs are diff-able and machine-checkable. Delete guidance italics, keep the
`##` headings even when a section is empty (write `None.`).

---

# <Feature Name>

- **Spec ID:** `<AREA-NNN>` (e.g. `TOOL-014`, `FILT-003`, `ARCH-002`)
- **Status:** `Stub | Draft | Spec'd | Verified`
- **Parity tier:** `Core | Extended-only | Non-goal (Linux)` — CS6 Standard vs Extended and whether we target it.
- **New in CS6:** `Yes | No | Changed` + one line on what changed from CS5.
- **Depends on:** other Spec IDs / architecture docs.

## CS6 behavior

What the feature does, as the user experiences it. Exact tool names, panel names,
menu paths (`Filter > Blur > Gaussian Blur`), and observable behavior. Call out
edition differences. This section is the contract; if it is vague, the spec is
unfinished.

## UI surface

Every place the feature appears: menu item, tool slot, panel, dialog, context
menu, keyboard shortcut, options bar. Table form where possible.

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| | | | |

## Parameters & ranges

Every user-visible control, its type, default, min/max, units, and step.

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| | | | | |

## Algorithms & pipeline

How it works under the hood, to the depth CS6 documents or the community has
publicly documented. Reference standard algorithms (e.g. "Gaussian = separable
FIR", "blend modes per PDF spec"). Mark anything inferred vs. sourced. Where the
exact Adobe implementation is closed, state "behavioral parity only, algorithm
TBD" — do not guess.

## Rust module mapping

Proposed crate/module layout for a Rust implementation. Names are provisional
but concrete. Note the data types crossing the boundary.

- `crate::module::Thing` — responsibility, key types.

## Qt6 component mapping

Proposed Qt6 widgets / QML types / models. Widgets vs. QML decision rationale.

- `QSomething` / `ThingView.qml` — responsibility.

## Data-model impact

Document-model fields, new node types, serialization (PSD keys, XMP), undo
granularity, and undo record shape.

## Edge cases

Boundary conditions, 8/16/32-bit behavior, CMYK/Lab behavior, empty/1-px
documents, huge (PSB) documents, GPU-unavailable fallback, undo/redo, memory.

## Parity acceptance criteria

Testable statements. "Given X, doing Y produces Z within tolerance T."
Each criterion should be checkable by a human or an automated comparison harness
later.

## Sources

Exact URLs actually fetched. Mark inferred/community sources as such.

- `<url>` — what it established.

## Open questions

Unknowns, contradictions between sources, decisions deferred. Every open
question must name what would resolve it.
