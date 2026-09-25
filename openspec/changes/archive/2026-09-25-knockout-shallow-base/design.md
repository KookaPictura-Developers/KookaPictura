# Design: knockout-shallow-base

## Context

`composite_layers` builds one `base` (the bottom layer composited alone) and
threads it to `composite_layer`, which routes any non-`None` knockout to
`composite_knockout(.., base)`. Pass-through groups forward that base to their
children; isolated groups build an empty `ko_base`. This makes Shallow and Deep
identical.

`psd-tools` 1.19.0 (`psd_tools/composite/composite.py`) resolves the base by mode:
`_knockout_backdrop(knockout)` returns the compositor's own `_color_0/_alpha_0`
for SHALLOW, and the lazily-resolved document Background for DEEP, falling back
to `_color_0/_alpha_0` when there is none. A pass-through group's child
compositor is seeded with the parent's running color/alpha (its `_color_0`) and
inherits the document-backdrop resolver; an isolated group's child compositor
resets that resolver to `None`, so DEEP there also falls back to the group's own
`_color_0`.

## Goals / Non-Goals

**Goals**

- Shallow stops at the enclosing group's entry backdrop; Deep still reaches the
  document Background, inherited through pass-through groups and reset at
  isolation.
- Match the psd-tools 1.19 nested pass-through Shallow reference.

**Non-Goals**

- A group layer's own knockout (the group composited as a source with `knockout`
  on the parent side), clipping-mask compositing, and `clbl`/`infx` semantics.

## Decisions

**Two bases.** Replace the single `base` parameter with
`deep: Option<&Canvas>` and `shallow: Option<&Canvas>`. Effective base for a
layer: `None` mode → no knockout; `Deep` → `deep.or(shallow)`; `Shallow` →
`shallow`. `composite_layer_inner` forwards both to its children and calls the
`composite_knockout` path only for the layer's own mode.

**Root.** `composite_layers` passes the same document-background canvas as both
`deep` and `shallow`, so root Shallow == Deep.

**Pass-through group.** When the group subtree contains a knockout, snapshot the
running `canvas` at group entry and pass it as `shallow` to the children; `deep`
is forwarded unchanged. This is exactly psd-tools' `_color_0` for the child
compositor.

**Isolated group.** Children get `deep = None` (falls back to `shallow`) and
`shallow = the group's empty initial backdrop` (today's `ko_base`), matching
psd-tools resetting the document backdrop and seeding alpha 0.

**Module split.** `composite.rs` is at the 1217 allowlist ceiling. Move the
knockout entry points (`composite_layers`, `knockout_base`, `has_knockout`,
`composite_layer`, `composite_knockout`) into a new `composite_knockout.rs`
that calls back into `composite_layer_inner` (made `pub(crate)`), keeping
`composite.rs` under budget and behavior identical.

## Risks / Trade-offs

- [Recursion signature change touches every composite path] → the only callers
  are inside the module; run the full workspace suite and the existing three
  knockout oracles (Deep root/group/isolated) plus the new Shallow one.
- [Canvas snapshot cost] → only when the group subtree actually contains a
  knockout (`has_knockout`), same guard as today.
- [psd-tools behavior drifts] → pin `psd-tools>=1.19` in CI and commit the
  generated reference.

## Migration Plan

Behavior change limited to Shallow inside groups; root Shallow is unchanged.
Revert is a revert.
