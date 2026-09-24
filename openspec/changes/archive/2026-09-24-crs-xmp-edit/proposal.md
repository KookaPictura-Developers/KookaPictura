# Proposal: crs-xmp-edit

## Why

Roadmap G15: Camera Raw settings for a raw opened as a Smart Object live in `crs:` XMP inside the embedded payload. Today `extract_crs` only exposes the packet on `SmartObject.crs_xmp`; the spec forbids an edit API, and the preserved `lnk*` section is re-emitted byte-exact so nothing can change.

## What Changes

- **Typed read of a small `crs:` property set** from the exposed packet: the common PV2012 tone/colour scalars (`Exposure2012`, `Contrast2012`, `Highlights2012`, `Shadows2012`, `Whites2012`, `Blacks2012`, `Clarity2012`, `Vibrance`, `Saturation`, `Temperature`, `Tint`) as `Option<f64>` attributes on `SmartObject`. Unknown `crs:` keys stay inside the raw packet (preserve-only).
- **Edit API** `set_crs_property(&mut Document, uuid, name, value)`: span-patches one `crs:Name="…"` attribute in the packet (same “replace only the matched span” discipline as `patch_xmp`), writes the new packet into both `SmartObject.crs_xmp` and `SmartObject.payload`, and rewrites the matching `liFD` payload bytes inside `Document.layer_section_extra` so open→save emits the edit. No `rdf:Description` / unpatchable construct → `Err` and no mutation.
- **Unmodified open→save stays byte-identical** (raw section path unchanged when the API is never called).
- **MODIFIED** `psd-smart-objects`: replace “SHALL NOT provide an API to edit `crs:`” with the edit requirement above; keep byte-exact preserve when unedited.
- **No UI**, no ACR dialog, no Camera Raw Filter (`Fltr`) changes (that path already has `set_camera_raw_option`).
- **BREAKING**: none for callers that never edit; the preserve-only contract becomes preserve-until-edited.

## Capabilities

### New Capabilities

<!-- None: this MODIFIED an existing preserve-only requirement. -->

### Modified Capabilities

- `psd-smart-objects`: `crs:` settings expose a typed subset and an in-place edit that round-trips through the embedded payload and the preserved `lnk*` section.

## Impact

- `crates/pictura-core`: optional typed `crs` view fields on `SmartObject` (or a nested `CrsSettings` struct).
- `crates/pictura-codec`: span patcher for `crs:` attributes; `set_crs_property`; splice into `layer_section_extra`.
- Tests: synthetic embedded payload with `crs:Exposure2012` (extend the existing unit test); edit → write → re-read; unmodified fixture still byte-identical; patch failure leaves the document unchanged.
- No new dependency; no app UI.
