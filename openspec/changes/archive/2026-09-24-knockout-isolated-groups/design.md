# Design: knockout-isolated-groups

## Context

`crates/pictura-render/src/composite.rs::composite_layer_inner`'s isolated
branch builds `inner = Canvas::new_region(...)` (transparent), composites the
group's children into it with `base = None`, then `composite_canvas(canvas,
layer, &inner)` composites the group over the running backdrop.

An isolated group's initial backdrop is that empty `inner` before any child
runs. `psd-tools`' compositor stops deep knockout at an isolated group
(`composite.py`, "escapes pass-through groups but stops at an isolated one"), so
the group's knockout child should base on that empty backdrop.

Probed psd-tools: red Background + yellow layer + isolated (Normal) group of
green and half-fill blue `knko = Deep` → `(126, 127, 128)`; pass-through gives
`(126, 0, 128)`. The CPU's current inert isolated result is `(0, 127, 128)`.

## Goals / Non-Goals

**Goals:**

- A knockout child of an isolated group punches through the group's earlier
  children to the group's initial backdrop, matching the psd-tools reference.
- Pass-through groups and the document root are unchanged.

**Non-Goals:**

- A knockout on the group layer itself.
- Shallow stopping points beyond the innermost enclosing group, clipping bases,
  `Transparency Shapes Layers`.

## Decisions

### D1. Isolated branch passes an empty initial backdrop

In `composite_layer_inner`'s isolated branch, when `has_knockout(layer)`, build
a second empty `Canvas::new_region(...)` (`ko_inner`) and pass
`Some(&ko_inner)` to `composite_layer` for the children; otherwise pass `None`
(unchanged, so a knockout-free group is byte-identical and pays no allocation).
The incoming `base` (the document background) is intentionally ignored inside an
isolated group: the group is its own knockout boundary.

### D2. Pass-through unchanged

The pass-through branch still propagates the incoming `base`; only the innermost
enclosing group decides whether the base is the document background or the
group's initial backdrop.

### D3. Fixture and reference

`generate-fixtures.py` gains `knockout_isolated_group()`: red Background, a
yellow layer, then an isolated group (explicit non-pass blend, e.g. Normal)
containing green and half-fill blue `knko = Deep`. The reference comes from the
existing `psd_knockout_reference.py gen --out`. The oracle test asserts the
decoded group blend is not `PassThrough` and the child is `Deep`, compares at
tolerance 1, and asserts the discriminating signature (the group's green is
punched through while the yellow below still contributes).

### D4. Tests

Unit tests in `composite.rs`: an isolated-group knockout punches the group's
green through to a transparent initial backdrop; a knockout-free isolated group
is byte-identical; the document-root and pass-through tests stay green.

## Risks / Trade-offs

- [Coincidental equality] → the fixture includes a layer below the isolated
  group (yellow) so the isolated base is genuinely discriminated from the
  document background; the previous pass-through fixture could not.
- [Rounding] → tolerance 1, as before.
- [Spec churn] → the same requirement is MODIFIED a third time; accepted because
  the behavior genuinely extends.
