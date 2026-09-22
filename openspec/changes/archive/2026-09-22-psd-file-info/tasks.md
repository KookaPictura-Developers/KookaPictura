## 1. Engine: EXIF and IPTC decoders

- [x] 1.1 Add `crates/pictura-codec/src/exif.rs`: `Exif`, `ExifValue`, `parse_exif`, `get`; II/MM byte order, IFD0 + Exif sub-IFD (`0x8769`), types ASCII/SHORT/LONG/RATIONAL/UNDEFINED, `Exif\0\0` prefix or bare TIFF, bounded per-IFD entries, no panic on truncation.
- [x] 1.2 Add `crates/pictura-codec/src/iptc.rs`: `Iptc`, `parse_iptc`, `get`; `0x1C` record/dataset markers, big-endian `u16` length, stop-and-return on malformed input, bounded record count.
- [x] 1.3 Add unit tests in each module for the happy path, big-endian, and truncated input (no panic), using hand-built byte blobs.
- [x] 1.4 Add `crates/pictura-codec/src/metadata.rs`: `DocumentMetadata`, `read_metadata(&Document)` that finds resources 1058/1059, 1028, 1060 and exposes the XMP packet as UTF-8-lossy text.
- [x] 1.5 Register the modules and `pub use` the public types/functions in `crates/pictura-codec/src/lib.rs`; add a unit test covering a document with and without metadata resources.

## 2. Fixture and oracle

- [x] 2.1 Add a `metadata()` builder to `scripts/generate-fixtures.py` that authors `metadata.psd` with a real EXIF IFD (IFD0 + Exif sub-IFD), an IPTC-IIM block (1028), and an XMP packet (1060), from fixed byte constants; register it in `FIXTURES` and regenerate.
- [x] 2.2 Add the fixture to `crates/pictura-codec/tests/oracle/support.rs` and a `metadata_oracle.rs` integration test that compares `read_metadata` against `exiftool -j -G1 -s`, self-skipping when `exiftool` is absent.
- [x] 2.3 Update `crates/pictura-codec/tests/fixtures/README.md` with the new fixture.

## 3. App: File Info dialog

- [x] 3.1 Add `crates/pictura-app/cpp/file_info_dialog.{h,cpp}`: read-only `QDialog` with a category list (Camera Data, IPTC, Raw Data) and a stacked pane, taking the rows/packet as constructor arguments, with `*ForTest` accessors; register both files in `CMakeLists.txt`.
- [x] 3.2 Add `command_ids::FileInfo = "file.fileInfo"`, promote the `File > File Info…` leaf in `command_tree.cpp`, and add a `frame_menus.cpp` handler gated on an open document.
- [x] 3.3 Add `exif_rows`, `iptc_rows`, and `xmp_packet` `#[qinvokable]` getters to the bridge (`cxxqt_object.rs`) with impls in `impl_core.rs`, using `pictura_codec::read_metadata`; add a `showFileInfo()` handler in `frame.cpp`.
- [x] 3.4 Add a self-test (next free code, `456`) that opens the metadata fixture, asserts the decoded EXIF/IPTC rows, and constructs the dialog to check its categories and rows.

## 4. Verification

- [x] 4.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace` and `cargo test --workspace --doc` are green.
- [x] 4.2 `bash scripts/verify-fast.sh` is green (or `verify-full.sh` after a CMake build); `openspec validate psd-file-info --strict` passes.
- [x] 4.3 Confirm no file exceeds its size cap and `scripts/guard.sh` passes.
