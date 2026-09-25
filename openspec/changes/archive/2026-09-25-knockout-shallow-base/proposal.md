# Proposal: knockout-shallow-base

## Why

`composite.rs` treats `Knockout::Shallow` and `Knockout::Deep` identically: it
threads one `base` (the document background) and uses it for both. Photoshop's
documented semantics differ inside a group — **Shallow** stops at "the first
possible stopping point … the first layer after the layer group", i.e. the
backdrop current when the enclosing group began, while **Deep** reaches the
document Background. `psd-tools` **1.19.0** now implements and discriminates
this (`psd_tools/composite/composite.py`, `_knockout_backdrop`: SHALLOW →
`_color_0/_alpha_0`; DEEP → the document backdrop, inherited across pass-through
groups and reset at isolation), verified against Photoshop-authored fixtures, so
the previously-ungated nested-Shallow case now has an oracle.

## What Changes

- The compositor SHALL thread two bases, `deep` (document background) and
  `shallow` (the initial backdrop of the compositor applying the layer), and
  select by mode: `Deep` → `deep`, falling back to `shallow` when there is no
  Background; `Shallow` → `shallow`.
- At the document root the two coincide, so Shallow still equals Deep there.
- Inside a pass-through group, a child's `shallow` SHALL be the backdrop current
  when the group began; `deep` continues to be inherited.
- Inside an isolated group, `deep` SHALL reset to the group's own initial
  backdrop (the existing behavior), and `shallow` SHALL be that same backdrop.
- A new committed fixture plus a `psd-tools`-derived reference SHALL cover the
  nested pass-through Shallow case; the CI `psd-tools` pin SHALL move to
  `>=1.19` so the oracle that discriminates Shallow cannot silently regress.
- Clipping-mask knockout targets remain out of scope (the compositor does not
  composite clipping masks, and psd-tools cannot discriminate them).

## Capabilities

### Modified Capabilities

- `knockout-compositing`: distinguishes Shallow from Deep, and adds a
  nested-Shallow oracle fixture.

## Impact

- `crates/pictura-render/src/composite.rs` (two-base threading) — the file is at
  its 1217-LOC allowlist ceiling, so the knockout helpers SHALL move to a new
  module (`composite_knockout.rs`) to stay within budget; `composite.rs` must not
  grow past 1217.
- `scripts/generate-fixtures.py` (new `knockout_shallow_group` fixture),
  `scripts/psd_knockout_reference.py` (its reference), `knockout_oracle.rs`
  (the comparison test), `.github/workflows/ci.yml` (pin `psd-tools>=1.19`).
- No new dependency; no PSD read/write change.
