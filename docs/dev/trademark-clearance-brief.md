# Trademark clearance and nominative-use brief

- **Status:** engineering input for counsel. **Not legal advice.**
- **Purpose:** record every Adobe mark used anywhere in the project, classify it
  as nominative/format-identifier vs. product/feature label, and scope a
  trademark-clearance request.

## New marks to clear (for counsel)

- **"Kooka Pictura"** — product name.
- **"KookaPictura-Developers"** — GitHub organisation name.
- Requested action: knock-out + full clearance search and (if clear) filing in
  **Nice classes 9** (software) and **42** (software services) for the relevant
  jurisdictions; assess likelihood of confusion with existing marks.

## Audit method

Searched shipped C++ UI strings (`crates/pictura-app/cpp/**`), Rust
user-facing strings (`crates/**/*.rs`), `NOTICE.md`, `README.md`, and the About
box for `Adobe`, `Photoshop`, `Camera Raw`, `Lightroom`, and Adobe-coined
feature names.

## Findings and disposition

### 1. Product/feature labels using Adobe-coined names — attention

| String | Where | Disposition |
|---|---|---|
| "Content-Aware Scale" | `command_tree.cpp` | **Rebranded** to "Seam-Aware Scale" (shipped UI); the spec keeps the Photoshop name nominatively (`docs/02-ui-ux/menus.md`). |
| "Puppet Warp", "Smart Object", "Smart Filter", "Refine Edge", "Vibrance", "Quick Selection", "Lens Correction", "Color Lookup", "Photo Filter", "Channel Mixer" | menu labels (`command_tree.cpp`, `frame_menus*`) | **Pending product decision + counsel.** These are Adobe-coined feature names used as our labels. The licensing policy prefers descriptive names; renaming the whole tree affects the CS6 menu spec, self-tests, and user expectations, so it is not done unilaterally. |

Most other menu labels are generic/descriptive and used industry-wide
(Levels, Curves, Brightness/Contrast, Hue/Saturation, Gaussian Blur, Unsharp
Mask, Emboss, Mosaic, …).

### 2. Nominative / format-identifier uses — keep

| String | Where | Why |
|---|---|---|
| "Adobe RGB (1998)" | `profile_dialog.cpp` (colour-profile combo) | Standard colour-space name; identifies the space. |
| "Camera Raw Filter" (`CAMERA_RAW_FILTER_NAME`) | `pictura-codec/src/pictura_raw.rs` | On-disk filter name Photoshop writes into `SoLd.filterFX`; required for round-trip. |
| "Adobe Camera Raw Filter" class ID | same | PSD descriptor format identifier. |
| Smart-filter id `2683` | same | Numeric PSD format identifier. |
| XMP namespaces `adobe:ns:meta`, `ns.adobe.com/photoshop/1.0/` | `pictura-codec` | Format namespaces reproduced byte-for-byte. |
| "AdobeInvisFont", "MyriadPro-Regular" | EngineData fixtures/tests | Data values in a text-layer blob; not our labels. |
| "Photoshop CS6" | docs/spec corpus | Nominative reference to the compatibility target. |

### 3. Artwork and trade dress

No Adobe logos, icons, cursors, or trade dress: all 238 SVG icons/cursors are
original (see `assets/PROVENANCE.md`). Feature/profile/brush presets are not
bundled.

### 4. Disclaimer

Present in [`NOTICE.md`](../../NOTICE.md), `README.md`, `docs/README.md`, and
the **About** dialog: independent project, not affiliated with or endorsed by
Adobe; Adobe marks used only nominatively.

## Requests for counsel

1. Clear and, if possible, register **"Kooka Pictura"** / the org name (classes
   9/42).
2. Confirm the nominative-use posture for the "Photoshop CS6" compatibility
   references and the disclaimer wording.
3. Decide the **feature-label policy**: may we keep Adobe-coined feature names
   (Smart Object, Refine Edge, Puppet Warp, Vibrance, …) as descriptive
   compatibility labels, or must they be renamed? Provide a list-based rule.
4. Confirm whether the format identifiers in §2 are safely reproduced.
