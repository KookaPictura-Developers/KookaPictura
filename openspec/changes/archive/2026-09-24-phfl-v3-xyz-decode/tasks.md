# Tasks: phfl-v3-xyz-decode

## 1. Decode

- [x] 1.1 In `crates/pictura-render/src/composite.rs::decode_photo_filter`, accept version 3: read three `be_u32` XYZ fields at offsets 2/6/10, density at 14, luminosity at 18; keep version-2 path unchanged.
- [x] 1.2 Add a private (or codec-exported) `xyz_d50_to_srgb_u8` using the same D50→sRGB matrix as `pictura-codec`'s `lab_to_rgb`, 16.16 scale (`/ 65536.0`), clamp to `0..=255`; `ponytail:` comment naming the unproven-scale ceiling.
- [x] 1.3 Reject as `None`: truncated payload, version ∉ {2,3}, density > 100 (both versions).

## 2. Tests

- [x] 2.1 `crates/pictura-render/src/tests/adjustment/part_decode.rs`: replace the v3 case in `deferred_keys_still_none` with a v3 decode test (`phfl_decodes_version_three`); drop any v3→`None` assert inside `phfl_decodes_version_two`.
- [x] 2.2 Malformed v3 cases: truncate before luminosity, density 101 → `None`.
- [x] 2.3 Encoder round-trip: decoded v3 params → `encode_photo_filter` → decode equals params (colour within 8-bit quantisation if XYZ conversion is lossy).
- [x] 2.4 Run: `cargo nextest run -p pictura-render adjustment` and confirm the Photo Filter composite tests still pass.

## 3. Spec / gates

- [x] 3.1 `openspec validate phfl-v3-xyz-decode --strict`
- [x] 3.2 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 3.3 Confirm the delta still validates after code lands: `openspec validate phfl-v3-xyz-decode --strict` (deferred requirement is REMOVED, not MODIFIED).
- [ ] 3.4 Optional: add a psd-tools-written v3 `PhotoFilter` fixture assertion in codec tests only if it does not churn `adjustment.psd`; otherwise skip (unit layout is enough).
