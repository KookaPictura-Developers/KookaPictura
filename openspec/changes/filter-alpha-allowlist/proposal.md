# Proposal: filter-alpha-allowlist

## Why

Issue #248. On an unlocked layer, `apply_op_region` reruns the filter over a
grey copy of the alpha plane and writes the result back as transparency. That
is only right for a kernel that maps a flat 255 plane back to 255. A
blocklist (`filter_preserves_opacity`) exempted four kinds; every other
value-dependent or plane-replacing kernel rewrote opacity: Solarize turned an
opaque layer fully transparent, Clouds left the cloud tone as alpha, and Offset
without wrap cleared the whole layer with a black Background.

## What Changes

- `filter_preserves_opacity` becomes an allowlist: only kernels that move,
  spread, or rank samples independently of their value (blurs, sharpens, rank
  filters, Offset, Mosaic, Crystallize, Fragment, the geometric Distort family)
  run the alpha pass. Every other kind leaves alpha bit-identical.
- Offset's alpha pass fills the exposed area with 255: the dialog's single
  `Background:` colour is CS6's "Set to Background", which paints opaque on a
  normal layer. "Set to Transparent" stays unexpressible (follow-up).
- The `filter-application` alpha requirement, which still read "MUST NOT be
  modified", states the actual contract.

## Capabilities

### Modified Capabilities

- `imaging/filter-application`: the alpha requirement names which kinds filter
  an unlocked layer's transparency and which leave it bit-identical.

## Impact

`crates/pictura-render/src/filter.rs` only. No bridge, dialog, or dependency
change.
