# M4-B — Adjustment Layers

Goal: adjustment layers work end to end — a layer that carries an adjustment,
applied to everything below it, gated by its mask/opacity/blend, and preserved
through PSD read/write.

Depends on M4-A (`pictura-adjust`). Spec: `docs/05-layers/adjustment-layers.md`,
`docs/04-image-ops/adjustments/*.md`.

## Design constraints

- `pictura-core` must **not** depend on `pictura-adjust` (layering). The model
  carries the adjustment **opaquely**:
  ```rust
  pub struct AdjustmentData { pub key: [u8; 4], pub data: Vec<u8> } // raw PSD bytes
  // Layer gains: pub adjustment: Option<AdjustmentData>,
  ```
- `pictura-codec` reads/writes the adjustment additional-layer-info block
  (`'levl'`, `'curv'`, `'brit'`, `'expA'`, `'vibA'`, `'hue2'`, `'blwh'`, `'phfl'`,
  `'mixr'`, `'grdm'`, `'invr'`, `'post'`, `'thrs'`, `'selc'`, `'clrL'`) and keeps
  the bytes verbatim.
- `pictura-render` depends on `pictura-adjust` and **decodes** an
  `AdjustmentData` into an `Adjustment` for the subset it understands. Unknown or
  undecodable keys are a **no-op** (preserved on save, not applied) — never an error.
- Applying an adjustment layer: apply the adjustment to the running backdrop
  (the accumulated composite below the layer), then gate the result by the layer's
  mask/opacity/blend. (Photoshop applies the adjustment to the backdrop and blends
  the adjusted result back.)

## Scope

In:
- Model + codec raw round-trip for adjustment layers.
- A decoder for a practical subset of PSD adjustment encodings, with tests.
- Compositor application of a decoded adjustment layer.
- If psd-tools can author adjustment-layer PSDs, use them as fixtures; otherwise
  hand-build the bytes and document.

Out (later):
- Full descriptor coverage for every adjustment; adjustment-layer clipping
  subtleties; Color Lookup / Gradient Map / Selective Color decoding.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M4-B | model + codec + render + tests | agent | `crates/pictura-core`, `crates/pictura-codec`, `crates/pictura-render` |

## Exit gate

- `cargo test --workspace` green.
- An adjustment layer round-trips through `write_psd`/`read_psd` with its key+bytes intact.
- A decoded adjustment layer (e.g. Invert) applied over a pixel layer produces
  the same result as applying that `Adjustment` to the flattened composite,
  within ±1; its mask/opacity gate the effect.
- `scripts/guard.sh` green.
