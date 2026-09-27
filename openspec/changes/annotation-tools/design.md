# Design: annotation-tools

## Decisions

**Document state vs view state.** Color samplers and notes are saved with the
image in CS6, so they live on `Document::annotations` and ride the history
snapshot like slices. Every committed add, move, delete, clear, or note edit is
one history state; CS6's undo behaviour for samplers is unverified
(`docs/03-tools/eyedropper-color-sampler-ruler.md`), and recording them keeps
an unrelated undo from silently reverting them. The Ruler's line is not saved
and not undoable in CS6, so it is held on the view (`PictureViewRust::ruler`).
`ponytail:` samplers and notes are not yet written to or read from the PSD
(resource 1073, the `Anno` block), which stays preserved verbatim.

**Sampler cap.** CS6 allows four samplers (photorust models the later ten); a
fifth is refused with a status-bar message, not evicted.

**Ruler angle.** A is anticlockwise from east as seen on screen, so a line
running down-right reads negative (photorust and CS6). The spec's acceptance
example (0,0)→(3,4) gives |A| = 53.13°; the sign follows CS6.

**Interaction.** Samplers and notes are placed at the rounded pointer position,
the same pixel the Eyedropper and Info panel read. They are grabbed within
8 screen pixels; a drag moves one live and records one state on release; a drag
off the canvas or Alt-click deletes. The Ruler grabs the nearer end, or the
line to move it; a click without a drag leaves no line. Samplers and notes are
drawn with every tool (CS6's Extras); the Ruler's line only while it is active.

**Notes panel.** CS6 edits note text in the Notes panel, not a dialog, so the
placeholder panel becomes a real one: the tool controller owns the current
note, the panel shows it, commits typed text as one "Edit Note" state on focus
loss or note change, and steps previous/next or deletes.

**One handler for two tools.** Color Sampler and Note differ only in marker
kind and the note opening, so one `MarkerToolHandler` serves both.

## Non-Goals

- Sample Size for samplers, Eyedropper Shift-click to add one, per-sampler
  readout colour spaces.
- The Ruler's protractor (Alt-drag), Straighten, and units other than pixels.
- Note Author and Color, the Count tool, and `View > Show > Notes`.
