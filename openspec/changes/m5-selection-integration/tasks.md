## 1. Document channels

- [x] 1.1 Add `Document.channels: Vec<Channel>` in `pictura-core`, initialised empty by `Document::new`

## 2. PSD extra-channel codec

- [x] 2.1 Write `color_channels + doc.channels.len()` in the header channel count
- [x] 2.2 Append extra channel planes after the color planes in the image-data section
- [x] 2.3 Split extra planes into `Document.channels` on read while keeping the composite to color planes
- [x] 2.4 Reject extra-channel length mismatches on write
- [x] 2.5 Add round-trip tests for one and multiple extra channels plus the length-mismatch case
- [x] 2.6 Add the psd-tools oracle test that reads the written extra channel

## 3. Selection ↔ channel conversion

- [x] 3.1 Add `Selection::to_channel(id)` in `pictura-select`
- [x] 3.2 Add `Selection::from_channel(&Channel, width, height)` with size-mismatch error
- [x] 3.3 Add round-trip and wrong-length tests

## 4. App selection state

- [x] 4.1 Add `PictureView` selection field and `select_all()`, `deselect()`, `magic_wand(x, y, tolerance)`
- [x] 4.2 Expose `has_selection()` and `selection_count()`, emitting `changed` on selection updates
- [x] 4.3 Build `selection_to_mask` and attach it in `add_adjustment` when a selection is active

## 5. App UI

- [x] 5.1 Add Select all / Magic wand (center) / Deselect buttons wired to the view
- [x] 5.2 Display the current selection pixel count and refresh after changes

## 6. Verification

- [x] 6.1 Add the `--self-test` wand-select → Invert → confinement check and exit non-zero on failure
- [x] 6.2 Run `cargo test --workspace` and the psd-tools oracle
- [x] 6.3 Run the headless self-test against a layered PSD
