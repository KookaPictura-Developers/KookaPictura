## Context

`apply_icc` (`crates/pictura-codec/src/icc.rs:29`) runs unconditionally from the
two `read_psd` return paths (`read.rs:113`, `:181`): for an RGB document with a
decodable non-sRGB resource 1039 it converts the composite and every layer to
sRGB, records `source_icc`, and drops 1039. `Document.document_icc` (the working
profile that drives the display conversion `buffer_to_srgb`, `icc.rs:223`) and
`assign_document_profile`/`convert_document` already exist from
`color-profile-assignment`.

`WF-011` and `ARCH-007` define the incoming policy (`Off`, `Preserve Embedded
Profiles`, `Convert to Working`) with Preserve the default, and state the
settings are global preferences, not history. The reader takes only a byte slice,
so the policy must be threaded in via a new entry point; `read_psd` has many
callers, including an engine-level read of embedded smart-object payloads
(`pictura-render/src/composite.rs:295`).

## Goals / Non-Goals

**Goals:**
- A policy-driven open: Preserve keeps the embedded profile and its pixels,
  Convert keeps today's behaviour, Off ignores the profile.
- The canvas shows the right appearance under Preserve without converting the
  stored pixels (reusing `document_icc` + `buffer_to_srgb`).
- The policy is a persisted application preference and is editable from
  `Edit > Color Settings…`.

**Non-Goals:**
- A configurable working space (sRGB only), CMYK/Gray policies, `.csf` files,
  and the mismatch/missing-profile dialogs.
- Changing `read_psd`'s default (Convert) or the nested smart-object read.
- Any change to Assign/Convert (they stay per-document undoable commands).

## Decisions

- **`read_psd_with(bytes, policy)`; `read_psd` stays Convert.** Keeping
  `read_psd` as `read_psd_with(bytes, Policy::Convert)` leaves every existing
  caller and test unchanged, including the engine's smart-object payload read
  and the ICC oracle; the application is the place that knows the user's
  preference and calls `read_psd_with`. A `// ponytail:` note records that the
  library convenience is Convert while the application default is Preserve.
- **Preserve is expressed through `document_icc`, not a pixel transform.** The
  embedded profile goes to `document_icc`, resource 1039 stays in
  `image_resources`, and pixels are untouched, so the save re-emits the file
  still tagged and `buffer_to_srgb` renders it correctly. This reuses the
  shipped working-profile machinery rather than adding a second conversion path.
- **`Policy` lives in `pictura-color`** alongside the profiles it selects.
- **The preference is persisted in the existing session store**
  (`SessionState` / `$XDG_STATE_HOME/kooka-pictura/state.json`) as an integer,
  matching how the app persists other prefs; the Rust view mirrors it like
  `gpu_compute` so `open` can pass it to `read_psd_with`.
- **A small Color Settings dialog** sets the RGB incoming policy; the working
  space is shown fixed as sRGB.

## Risks / Trade-offs

- **The application default flips to Preserve**, so opening a wide-gamut file in
  the app no longer auto-converts; this is the contract's intent but a visible
  behaviour change. The library default stays Convert for compatibility.
- **Preserve + display conversion** means the stored pixels are in the embedded
  space while the canvas shows sRGB; an edit made under Preserve operates in the
  embedded space (as Photoshop does), and the save keeps the tag. This is the same
  model as Assign Profile and inherits its ceilings.
- **Nested smart-object payloads** still read with Convert, so a smart object
  carrying its own non-sRGB profile is normalised when sampled even under
  Preserve. Marked `ponytail:`; a later change can thread the policy into the
  render read.
- **`xmp`-style `.csf` interop is not provided**; settings live in the app store.

## Migration Plan

Additive: a new entry point and a new preference defaulting to Preserve for the
app, Convert for direct `read_psd` callers. Existing specs' Convert behaviour is
unchanged, so no test migration beyond adding policy scenarios. No document-model
migration.

## Open Questions

- Whether the working space and the CMYK/Gray policies, or a real `.csf`, ship
  later; the contract leaves the `.csf` layout undocumented.
