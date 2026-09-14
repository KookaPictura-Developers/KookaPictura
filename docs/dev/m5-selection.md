# M5 — Selection & Mask Math

Goal: a selection is an 8-bit **coverage mask**; implement the algebra and
modify operations that every masked edit depends on.

Spec: `docs/08-selection/*.md` (selection-model, grow-similar-and-modify,
color-range, quick-mask, save-and-load-selections).

## Scope

In (new crate `crates/pictura-select`):
- `Selection` = document-sized 8-bit coverage (0 = outside, 255 = inside).
- Boolean ops: Replace / Add (union) / Subtract / Intersect.
- `invert`, `none`, `all`.
- Modify: `feather` (blur the mask), `expand`/`contract` (dilate/erode),
  `border`, `smooth`.
- Image-derived: `magic_wand` (flood by tolerance, contiguous/global),
  `grow`, `similar`, `color_range` (by color + fuzziness).
- `save`/`load` to/from an alpha `Channel` (PSD channel bytes).

Out (later):
- Refine Edge (smart radius, decontaminate), quick-selection brush heuristics,
  vector-mask boolean ops, per-channel 16-bit masks.

## Contract (M5-A authoritative — suggested)

```rust
pub struct Selection { pub width: u32, pub height: u32, pub data: Vec<u8> }
pub enum SelectOp { Replace, Add, Subtract, Intersect }

impl Selection {
    pub fn none(w: u32, h: u32) -> Self;
    pub fn all(w: u32, h: u32) -> Self;
    pub fn combine(&mut self, other: &Selection, op: SelectOp);
    pub fn invert(&self) -> Selection;
    pub fn feather(&self, radius: f64) -> Selection;
    pub fn expand(&self, radius: u32) -> Selection;
    pub fn contract(&self, radius: u32) -> Selection;
    pub fn border(&self, width: u32) -> Selection;
    pub fn smooth(&self, radius: u32) -> Selection;
}

pub fn magic_wand(img: &PixelBuffer, x: u32, y: u32, tolerance: u8, contiguous: bool) -> Selection;
pub fn grow(sel: &Selection, img: &PixelBuffer, tolerance: u8) -> Selection;
pub fn similar(sel: &Selection, img: &PixelBuffer, tolerance: u8) -> Selection;
pub fn color_range(img: &PixelBuffer, target: [u8;3], fuzziness: u8) -> Selection;
```

- Combine semantics per the CS6 model: `Add = max(a,b)`, `Subtract = saturating a-b`,
  `Intersect = a*b/255`, `Replace = b`.
- Deterministic; mismatched dimensions → error, never panic.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M5-A | `pictura-select` core | agent | `crates/pictura-select` |
| M5-B | ImageMagick morphology/feather oracle | agent | `scripts/**`, `crates/pictura-select/tests/**` |
| M5-C | Channel save/load + integration | orchestrator | — |

## Oracle

ImageMagick: `-morphology Dilate/Erode` (expand/contract), `-gaussian-blur`
(feather), `-threshold`/`-blur` (smooth). Tolerance documented; the wand and
color-range get known-value/property tests (no faithful IM equivalent).

## Exit gate

- `cargo test --workspace` green; boolean-op identities, modify ops, and wand
  tests pass.
- IM differential within tolerance for morphology/feather, or documented divergence.
- `scripts/guard.sh` green.
