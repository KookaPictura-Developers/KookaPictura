## Why

`WF-010` requires metadata templates: export a document's File Info fields as a
standalone `.xmp` file and apply one to another document, with photo-specific
camera data preserved. The engine can patch an existing packet but cannot
serialize a standalone one or merge a template, and File Info has no template
control, so the workflow is missing.

## What Changes

- Add `to_xmp_packet(&XmpProperties) -> String`: serialize the managed property
  set into a standalone, well-formed XMP packet (lists as `rdf:Seq`/`Bag`), which
  `parse_xmp` reads back to the same properties.
- Add `export_template(&Document) -> Vec<u8>`: the document's parsed XMP
  properties as a standalone packet to write to a `.xmp` file.
- Add `MergeMode { Append, Replace, KeepOriginalReplaceMatching }` and
  `apply_template(&mut Document, &XmpProperties, MergeMode) -> bool`:
  - **Append** sets a managed field only when it is currently empty/absent.
  - **Replace** sets every managed field from the template and clears the ones
    the template omits.
  - **KeepOriginalReplaceMatching** overwrites only the fields the template
    defines and never clears the others.
  Every mode patches the packet in place, so unknown namespaces, unknown
  properties, EXIF/camera data, and every other resource survive. The six
  IPTC-Core fields are also written to IIM so the two channels agree.
- File Info gains a Template control: Export (save a `.xmp`) and Apply (choose a
  `.xmp` and a merge mode), each recording one undo state.
- Ceilings: only the nine managed properties are templated; sidecars, a template
  folder/MRU, batch apply, and non-PSD template carriage are out of scope.
  Marked `ponytail:` in code.

## Capabilities

### New Capabilities
- `metadata-templates`: serialize a document's managed metadata to a standalone
  XMP template, and apply a template with Append / Replace /
  KeepOriginalReplaceMatching merge semantics, preserving camera data and unknown
  properties.

### Modified Capabilities
- (none — `psd-xmp-metadata` and `psd-file-info` behaviour is unchanged; the
  File Info dialog requirement does not forbid an added control)

## Impact

- `crates/pictura-codec`: a packet serializer in `xmp.rs` and a new
  `metadata/template.rs` merge layer plus `export_template`; re-exports in
  `lib.rs`. No new dependencies.
- `crates/pictura-app`: File Info Export/Apply controls, a merge-mode chooser,
  and bridge `export_metadata_template(path)` / `apply_metadata_template(path,
  mode)`; `cxxqt_object.rs` is at its 1200-line cap, so declarations need
  trimming. `CMakeLists.txt` gains any new files.
- Oracles: engine unit tests prove all three merge modes without external tools;
  a self-skipping exiftool check reads the applied fields back from a saved PSD.
