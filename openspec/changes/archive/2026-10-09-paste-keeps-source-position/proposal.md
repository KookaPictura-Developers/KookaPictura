# Proposal

## Why

Issue #99: a plain Edit ▸ Paste leaves the active marquee alive and re-centres
the clip on the canvas view, so copying a layer and pasting it does not put the
new layer where the source pixels sat. CS6 drops the selection after a paste,
and a copy/paste within a document lands in place.

## What Changes

- `clipboard_paste` (`crates/pictura-app/src/cxxqt_object/clipboard.rs`) places
  a plain Paste at the clip's own document rect — the origin Paste in Place
  already uses — instead of the view centre. Paste Into keeps centring on the
  selection bounds and Paste Outside on the view.
- Every successful paste drops the active marquee by moving the selection into
  `deselected_selection` (restorable with Select ▸ Reselect), not just the
  Into/Outside kinds.
- The `document/edit-clipboard` "Edit clipboard commands" requirement is
  modified; the Qt Test `tst_edit_clipboard` grows the plain-paste assertions.

## Capabilities

### Modified Capabilities

- `document/edit-clipboard`: plain Paste placement and the paste deselect.

## Impact

- `crates/pictura-app` bridge and Qt Test only. No engine, C++, or dependency
  change.
