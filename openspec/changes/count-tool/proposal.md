# Proposal: count-tool

## Why

The Count (Extended) tool (issue #9) was catalogued but disabled and tracked as
out-of-scope. It counts objects in an image with numbered marks. photorust
(perfecto25/photorust) ships it as an annotation (`MarkerKind::Count`) that
reads a document without changing its pixels. This change ports the core of it
onto Kooka's existing annotation infrastructure.

## What Changes

- `pictura_core::annotations`: a third marker list, `MarkerKind::Count`
  (numbered in placement order), independent of color samplers and notes.
- `tool_annotations.cpp`: a `Count` handler reusing the marker handler (click to
  add, drag to move, Alt-click to delete, Clear to reset); the canvas overlay
  draws each mark as a numbered disc.
- Catalog row enabled; options-bar Clear.
- C++ self-test `healing_tools` (code 536) covers the Count marks.

## Capabilities

### New Capabilities

- `tools/count-tool`: the Count (Extended) tool and its numbered marks.

## Impact

- `pictura-core` (`annotations.rs`), `pictura-app` bridge and C++ as above.
- No new dependency.

## Ceilings

`ponytail:` Count groups, per-group marker/label size and colour, the
Measurement Log, automatic counting, and PSD persistence are not shipped; marks
are session document state undone through history, like the other annotations.

## Provenance

Ported from photorust's `core/src/annotation.rs`
(<https://github.com/perfecto25/photorust>).
