## Why

`read_psd` unconditionally converts a non-sRGB embedded ICC profile to sRGB and
drops resource 1039, so opening a wide-gamut file destroys its profile. `WF-011`
requires an incoming-profile policy — default **Preserve Embedded Profiles** —
so the user chooses whether an incoming profile is preserved, converted, or
ignored. The `Edit > Color Settings…` menu entry exists but is a dead leaf.

## What Changes

- `pictura-color` gains `Policy { Off, Preserve, Convert }` for the incoming
  profile of an RGB document (the sRGB working space stays fixed).
- `pictura-codec` gains `read_psd_with(bytes, policy)`:
  - **Preserve** keeps the embedded profile: the pixels are not converted,
    resource 1039 is kept, and the document's working profile
    (`Document.document_icc`) is set so the canvas converts it to sRGB for
    display — the appearance is correct and the file re-saves still tagged.
  - **Convert** is today's behaviour: convert the composite and every layer to
    sRGB, drop 1039, record `source_icc`.
  - **Off** ignores the profile: pixels unchanged, 1039 dropped, untagged.
  A non-RGB, absent, sRGB, or undecodable profile is left unchanged as today.
- `read_psd(bytes)` keeps the Convert behaviour (the library convenience used by
  nested smart-object reads and existing callers); the application opens with the
  user's persisted policy via `read_psd_with`.
- The application persists the RGB incoming policy as an application preference
  (not history), defaults to **Preserve**, and `Edit > Color Settings…` opens a
  dialog to change it.
- Ceilings: sRGB working space only, RGB policy only, no `.csf` file, no
  mismatch/missing-profile dialogs (marked `ponytail:`).

## Capabilities

### New Capabilities
- `color-settings`: an incoming-profile policy (`Off` / `Preserve` / `Convert`)
  applied when a document is opened, persisted as an application preference, and
  exposed through `Edit > Color Settings…`.

### Modified Capabilities
- `color-profile-assignment`: the "a document has no working profile until
  assigned or converted" requirement changes — a document opened under the
  Preserve policy carries the embedded profile as `document_icc`.

## Impact

- `crates/pictura-color`: the `Policy` enum.
- `crates/pictura-codec`: `read_psd_with` and a policy-driven ICC pass in
  `icc.rs`; `read_psd` keeps Convert. No new dependencies.
- `crates/pictura-app`: a persisted `color_policy` preference, a bridge
  setter/getter, `open` passing the policy, and a real `Edit > Color Settings…`
  dialog (`cxxqt_object.rs` is at the 1200-line cap; new `.cpp/.h` need
  `CMakeLists.txt` entries).
- Tests/oracles: the existing ICC fixture proves Preserve (pixels byte-identical,
  1039 intact, `document_icc` set) and Convert (lcms2); Off untagged.
