## 1. Parser (`pictura-codec`)

- [x] 1.1 Add `crates/pictura-codec/src/image_resources.rs` with `pub struct ImageResource { id: u16, name: String, data: Vec<u8> }`, the id constants (`ICC_PROFILE = 1039`, `XMP_METADATA = 1060`, `EXIF_DATA_1 = 1058`, `EXIF_DATA_3 = 1059`, `IPTC_NAA = 1028`), and `pub fn decode_image_resources(document: &Document) -> Vec<ImageResource>`.
- [x] 1.2 Implement the block framer over `common::Reader`: accept `8BIM`/`8B64`/`MeSa`/`AgHg`/`PHUT`/`DCSR`, read the id, the even-padded Pascal name, the `u32` length, and the even-padded data; on any short read or unrecognized signature, stop and return the records so far.
- [x] 1.3 Re-export `decode_image_resources`, `ImageResource`, and the id constants from `crates/pictura-codec/src/lib.rs`.
- [x] 1.4 Unit tests: a hand-built section with two resources decodes to the ids/names/data in order; an odd-size name and data pad correctly; a truncated block returns the prior records and does not panic; an unrecognized signature stops; an empty section returns empty.

## 2. Fixture and oracle

- [x] 2.1 Add an `image_resources()` builder to `scripts/generate-fixtures.py` that saves a PSD with an image resource 1039 (fixed ICC-shaped bytes) and 1060 (a fixed XMP packet), register it as `image_resources.psd`, and commit it.
- [x] 2.2 Add a psd-tools oracle in `crates/pictura-codec/tests/oracle/` (or the existing `oracle.rs` submodule) that reads the fixture's resources with `psd_tools` and asserts each id/data equals `decode_image_resources`' output.
- [x] 2.3 Add a preservation check that an open→save of the fixture keeps the image-resource bytes identical (reusing the existing round-trip harness).
- [x] 2.4 Update `crates/pictura-codec/tests/fixtures/README.md` with the new fixture row.

## 3. Gates and commit

- [x] 3.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p pictura-codec`.
- [x] 3.2 `bash scripts/verify-full.sh` and `openspec validate --all --strict`.
- [x] 3.3 Confirm `scripts/check-file-size.sh` and `scripts/guard.sh` are green.
- [x] 3.4 Archive the change and commit the implementation, spec, fixture, and tests.
