# M5-C — Selection Integration

Goal: selections become usable — persisted as PSD alpha channels and able to
constrain an edit in the app.

## Scope

### C1 — Selection ↔ PSD alpha channels (codec + select)
- `pictura-core`: `Document.channels: Vec<Channel>` for **document-level alpha
  channels** (saved selections / spot channels), distinct from per-layer channels.
- `pictura-codec`: read/write the extra channels in the header channel count and
  the image-data section (after the color channels), so a saved selection
  round-trips. `read_psd`/`write_psd` must not regress composite/layer handling.
- `pictura-select`: `Selection::to_channel(id) -> Channel` and
  `Selection::from_channel(&Channel, w, h) -> Result<Selection>`.
- Oracle: psd-tools reads a written PSD and reports the alpha channel.

### C2 — Selection constrains an adjustment (app)
- `PictureView` selection state: `select_all()`, `deselect()`,
  `magic_wand(x, y, tolerance)`; the active selection is exposed for display.
- `add_adjustment(kind)` while a selection is active creates the adjustment
  layer **with a raster mask built from the selection** (so the effect is limited
  to selected pixels). No selection = full-frame effect (current behavior).
- App UI: buttons for "Select all", "Magic wand (center)", "Deselect".
- `--self-test`: after loading a layered PSD, wand-select one quadrant, add an
  Invert adjustment, and assert the inverted pixels are confined to that quadrant.

Out: Refine Edge, quick selection brush, adding a marquee tool, channel thumbnails UI.

## Acceptance

- `cargo test --workspace` green; a selection→PSD→selection round-trip test and a
  masked-adjustment parity test pass.
- psd-tools opens a written PSD and sees the alpha channel.
- `xvfb-run ./build/pictura --self-test <layered.psd>` proves a masked adjustment
  (only selected pixels change); exit 0.
- `scripts/guard.sh` green.
