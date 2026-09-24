# Design: multichannel-duotone-read

## Context

`color_mode_from_code` rejects codes 7 and 8. `ColorMode::{Multichannel, Duotone}` already exist; `color_channels()` is 1 for Duotone and 0 for Multichannel (header-authoritative). Adobe documents Duotone as “treat as grayscale, preserve the duotone info”; Multichannel has no composite RGB mapping and channel count varies.

## Goals / Non-Goals

**Goals:**

- Open Duotone and 1/3-channel Multichannel 8-bit documents into the RGB working space.
- Preserve Duotone `color_mode_data` and Multichannel plate bytes for unchanged write-back.
- Clear typed `Unsupported` for Multichannel with channel counts other than 1 or 3.

**Non-Goals:**

- Interpreting the duotone ink curves (spec stays opaque).
- Full plate-editing UI for Multichannel; 16/32-bit Multichannel/Duotone; inventing plate layout on edit.
- lcms/ICC color-managed CMY (profile-free only).

## Decisions

### D1. Duotone = grayscale + preserved spec

Accept mode 8. `color_channels()` is already 1. Normalize with `gray_to_rgb` (same as Grayscale). `retain_planes` for depth 8 Duotone; `source_mode = Duotone`; do **not** clear `color_mode_data`. Write-back: when `source_mode == Duotone` and planes unchanged (and flat/no extra-channel edits that would break the 1-plane layout), emit mode 8 + retained plane + original `color_mode_data`. Edited/layered → working RGB (existing fallback).

### D2. Multichannel channel-count gate

After header parse, if mode is Multichannel and `channels ∉ {1, 3}` → `PsdError::Unsupported`. Depth must be 8 (16/32 Multichannel stays unsupported).

### D3. Multichannel split uses header channel count as color count

For mode Multichannel only, `split_planes` / composite construction treat `header_channels` as the color-channel count (so all plates land in the composite, not extras). Then:

- N=1: `gray_to_rgb`.
- N=3: `cmy_to_rgb` = `(255-c, 255-m, 255-y)` per byte — **assumption**: 3-channel Multichannel from Photoshop’s RGB→Multichannel is CMY spots (Adobe scripting docs). Marked `ponytail:`; no lcms.

Layer color channels: Multichannel layers are rare; `convert_layer_color_channels` should apply the same mapping when `color_channels()` is patched for the read path, or skip if `color_channels()==0` (existing early return). Prefer: for Multichannel, convert using header N when N∈{1,3} on the document composite; layer conversion can stay a no-op for Multichannel if `color_channels()==0` (ceiling: Multichannel layers’ color ids may not convert — document it).

### D4. Write-back shape

Mirror Bitmap/Indexed: write mode 7 or 8 when `source_mode` is that mode and retained source planes still match the working buffer under the inverse of the read map (gray for Duotone/N=1; CMY inverse for N=3). If anything is edited or layers exist beyond a flat retained document, write working RGB. `color_mode_data` is always re-emitted from `doc.color_mode_data` (already the default path).

### D5. Fixtures

Hand-build minimal flat PSDs in unit tests (header + color-mode data + image data), same style as `flat_psd`/`header_depth` helpers. Optional psd-tools `PSDImage.new('Duotone'|'Multichannel')` round-trip if the helper can set image data without churning goldens.

## Risks / Trade-offs

- [Wrong Multichannel channel interpretation (not CMY)] → Only N=1 and N=3; CMY mapping marked ungrounded; other N refuse.
- [Write-back drops plates on edit] → Same class as Bitmap: unmodified flat only.
- [Duotone spec still opaque] → Preserve bytes; no curve modeling (roadmap G3 ceiling).

## Open Questions

- None blocking.
