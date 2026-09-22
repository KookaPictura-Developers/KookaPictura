## Why

The engine normalises an embedded profile to sRGB on read and drops resource
1039, and `Edit > Assign Profile…` / `Edit > Convert to Profile…` are dead menu
leaves. `IMG-006` (`docs/04-image-ops/color-profiles-and-assignment.md`) and
`ARCH-007` require the user to re-tag a document or transform its pixels between
profiles, as single undoable steps. Without a per-document working profile the
two operations cannot be told apart: assign leaves numbers untouched, convert
rewrites them.

## What Changes

- `Document` gains an optional working-profile ICC (`document_icc`): the profile
  the stored composite and layer color numbers are in. `None` means the sRGB
  working space, so every existing (read-normalised) document is unchanged.
- `Edit > Assign Profile…` retags the document with a chosen built-in profile
  (sRGB / Adobe RGB / Pro Photo RGB) without touching any pixel byte, as one
  undo step. The canvas appearance changes because the display conversion
  (`document_icc → sRGB`) now differs.
- `Edit > Convert to Profile…` transforms the composite and every layer's color
  channels from the current working profile to the chosen destination, then
  retags, as one undo step. Appearance is preserved; a lossy convert's undo
  restores the pre-conversion pixels.
- The composite shown on screen is converted from `document_icc` to sRGB at the
  render/display boundary; `doc.composite` and the layer channels stay in the
  document's profile so an assign does not destroy numbers.
- Saving tags resource 1039 with `document_icc`; a document with no/`sRGB`
  working profile still saves untagged (the existing read-normalisation path is
  unchanged).
- Ceilings: only the three built-in profiles are offered (no installed-profile
  discovery or `.icc` loading in the dialog); rendering intent is fixed to
  relative colorimetric with no black-point compensation, dither, or flatten;
  RGB 8-bit only (write support is RGB/Gray 8-bit). Marked `ponytail:` in code.

## Capabilities

### New Capabilities
- `color-profile-assignment`: a document carries a working ICC profile; the user
  can assign (retag, pixels untouched) or convert (transform pixels and retag)
  the document, with the display converted from that profile to sRGB and the
  profile tagged on save.

### Modified Capabilities
- (none — read-normalisation behaviour in `psd-icc-conversion` is unchanged: a
  freshly opened document still has no working profile)

## Impact

- `crates/pictura-core`: new `Document.document_icc: Option<Vec<u8>>` field
  (pictura-core has no dependencies, so the profile is stored as ICC bytes).
- `crates/pictura-codec`: new `assign_profile`/`convert_document` operations and
  a display-conversion helper, reusing the existing `convert_buffer` /
  `convert_layer`; the save path writes resource 1039 when a working profile is
  set. No new dependencies.
- `crates/pictura-render`: the composite returned for display is converted from
  the document profile to sRGB.
- `crates/pictura-app`: two real commands (ids, handlers, enabled providers), a
  profile-selection dialog, bridge methods, a C++ self-test check, and
  `CMakeLists.txt` entries. `cxxqt_object.rs` is at its 1200-line size cap.
- No new dependencies (rule 4); `pictura-render` already depends on
  `pictura-codec`, so the conversion routes through it rather than adding
  `pictura-color` to render.
