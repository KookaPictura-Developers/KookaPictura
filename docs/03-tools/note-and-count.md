# Note and Count Tools

- **Spec ID:** `TOOL-018` (Note); `TOOL-019` (Count)
- **Status:** `Draft`
- **Parity tier:** `Core` (Note); `Extended-only` (Count — Photoshop Extended, and the CS6 reference marks it "(Photoshop Extended only)")
- **New in CS6:** `No` — the Note tool and Count tool are carried over from CS5. The Count tool remains an Extended feature; both are documented in the CS6 reference.
- **Depends on:** `02-ui-ux/panels/note-panel.md`, `02-ui-ux/panels/measurement-log-panel.md`, `10-workflow-io/measurement-and-count.md`, `01-architecture/document-model.md` (`ARCH-002`), `01-architecture/undo-history.md`.

## CS6 behavior

### Note tool (`TOOL-018`)

The Note tool makes non-printing text notes attached to an image. Source: the
CS6 reference tools gallery (which describes notes attached to an image) plus
the Note tool procedure and Notes panel.

- Select the Note tool (grouped with Cursor/Count under Eyedropper;
  toolbox shortcut `I` cycle). Set **Author** and **Color** in the options bar.
- Click on the image to place a note icon. The **Notes panel** (`Window >
  Notes`) becomes active; type the note text there. The note persists as a small
  icon floating above the image and does not appear in print.
- Double-click a note icon with the Note tool to open/edit it in the Notes
  panel; use the panel's back/forward arrows to cycle through notes in the
  active image.
- Show/hide notes: `View > Show > Notes`, or `View > Extras` (which also
  toggles grids, guides, selection edges, target paths, and slices).
- Annotated documents can be saved in Photoshop (PSD), PDF, or TIFF formats.
- **Audio Annotation** (an audio counterpart) existed in earlier versions; its
  CS6 status is not established by the reference — see Open questions.

### Count tool (`TOOL-019`, Extended-only)

The Count tool counts objects in an image. Source: CS6 reference, "Counting
objects in an image (Photoshop Extended)".

- Select the Count tool (located beneath the Eyedropper tool in the Tools
  panel).
- **Count Group**: a default count group is created when the first count is
  added. Multiple groups are supported, each with its own name, marker size,
  label size, and color. The eye icon shows/hides a group; the folder icon
  creates a group; the Delete icon removes a group; `Rename` in the Count Group
  menu renames one. Clicks increment the currently selected group.
- **Color**: sets the count-group color via the color picker.
- **Marker Size**: 1–10 (scrubby slider). **Label Size**: 8–72 (scrubby slider).
- Click to add a count marker and label; numbering increments sequentially.
  Drag a marker/number to move it (`Shift` constrains horizontally/vertically).
  `Alt`/`Option`-click removes a marker and updates the total. **Clear** in the
  options bar resets the current group's count to 0.
- Show/hide counts: `View > Show > Count`, or `View > Extras` /
  `View > Show > All` / `View > Show > None`.
- **Automatic counting**: define selection areas with the Magic Wand tool or
  `Select > Color Range`, then count the multiple selection areas and record the
  result in the **Measurement Log** panel.
- **Record to Measurement Log**: `Analysis > Record Measurements` or the
  Measurement Log's Record Measurements button. The Count data point must be
  enabled first (`Analysis > Select Data Points > Custom` → Count tool area).
  Clearing counts on the image does not change counts already recorded in the
  Measurement Log.
- **Persistence**: `File > Save` saves count numbers and count groups added to
  the image.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Toolbox (`I` group) | Note tool | `I` cycles / hold Eyedropper | Core |
| Toolbox (`I` group) | Count tool | `I` cycles / beneath Eyedropper | Extended-only |
| Options bar (Note) | Author | — | Note author name |
| Options bar (Note) | Color | — | Note icon color |
| Panel | Notes (`Window > Notes`) | — | Text editing, prev/next notes |
| Menu | `View > Show > Notes` | — | Show/hide notes |
| Menu | `View > Extras` | — | Toggles notes with other Extras |
| Options bar (Count) | Count Group menu | — | New/eye/delete/rename groups |
| Options bar (Count) | Color | — | Group color |
| Options bar (Count) | Marker Size | — | 1–10 |
| Options bar (Count) | Label Size | — | 8–72 |
| Options bar (Count) | Clear | — | Reset current group to 0 |
| Menu | `View > Show > Count` | — | Show/hide counts |
| Menu | `Analysis > Select Data Points > Custom` | — | Enable Count data point |
| Menu | `Analysis > Record Measurements` | — | Write to Measurement Log |
| Panel | Measurement Log | — | Record Measurements button |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Note Author | String | System/empty | — | Metadata on the note |
| Note Color | Color | Default swatch | Color picker | Icon color |
| Note count | Count | 0 | Unlimited (per document) | Stored in PSD/PDF/TIFF |
| Count Group | Named list | "Count Group 1" (default) | Multiple groups | Per-group state |
| Group visibility | Bool | Visible | Eye icon | |
| Count Color | Color | Default | Color picker | Per group |
| Marker Size | Integer | e.g. 2 (default unverified) | 1–10 | Scrubby slider |
| Label Size | Integer | e.g. 12 (default unverified) | 8–72 | Scrubby slider |
| Count number | Integer | 1, incrementing | ≥ 0 | Per group |
| Record to log | Bool | Off | — | Requires Count data point |

## Algorithms & pipeline

- **Note**: a note is a document-space anchor plus text, author, color, and
  timestamp-ish metadata. It is rendered as a non-printing icon overlay and
  serialized with the document. Editing is plain text in the Notes panel.
- **Count**: each count is `{ group_id, index, point }`. Numbering is the
  next index within the group. Hit-testing selects the nearest marker within a
  tolerance for move/delete. Groups are independent counters.
- **Automatic counting**: connected-component analysis over the selection mask
  (Magic Wand / Color Range output); each connected component increments the
  count and a measurement record is appended to the Measurement Log.
- **Measurement Log record**: the Count data point captures the count (and,
  depending on enabled data points, source/scale/etc.). Clearing the on-image
  count does not rewrite existing Log rows.
- Behavioral parity only, algorithm TBD for the exact auto-count labeling and
  the Log row shape; the CS6 reference documents option semantics, not formats.

## Rust module mapping

Proposed:

- `pictura-core::annotations::Note` — `{ anchor: Point, text: String, author:
  String, color: Color, id: NoteId }`; `Vec<Note>` on the document.
- `pictura-core::annotations::NoteStore` — CRUD, iteration for the Notes panel,
  serialization hooks.
- `pictura-core::count::CountGroup` — `{ name: String, color, marker_size: u8,
  label_size: u8, visible: bool, marks: Vec<CountMark> }`.
- `pictura-core::count::CountState` — `Vec<CountGroup>`, active group,
  `add/remove/move/clear`, `auto_count(mask) -> Vec<CountMark>` via connected
  components.
- `pictura-core::measure::MeasurementLog` — `Vec<MeasurementRecord>`,
  `record(source: RulerCountSelection, data_points)` (shared with
  `10-workflow-io/measurement-and-count.md`).
- Boundary types: `NoteId`, `CountMark`, `CountGroup`, `MeasurementRecord`.

## Qt6 component mapping

- `NoteTool` — scene overlay drawing note icons; click to place, double-click to
  open.
- `NotesPanel` (`QDockWidget`/`QWidget`) — author/color display, multiline text
  editor, back/forward navigation, add/delete note.
- `CountTool` — scene overlay drawing markers and labels per group; drag/delete
  hit-testing; group color/marker/label rendering.
- `CountOptionsBar` (`QWidget`) — Count Group menu (new/eye/delete/rename),
  color picker, marker-size and label-size scrubby sliders, Clear.
- `MeasurementLogPanel` (`QDockWidget`) — table of records, Record Measurements
  button; shared with the measurement workflow.
- Widgets for panels (dense, model/view), scene items for canvas overlays.

## Data-model impact

- Document gains `notes: Vec<Note>` and `count_groups: Vec<CountGroup>` plus
  `active_count_group`, serialized so they survive save/reopen in PSD/PDF/TIFF
  (notes) and PSD (counts).
- Exact PSD keys TBD (Open questions). TIFF/PDF annotation channels carry notes.
- Undo granularity: each note add/edit/delete and each count add/remove/move/
  clear is a history state; group create/rename/delete likewise. Record shape:
  before/after list diff.
- Measurement Log rows are append-only and not undone by clearing counts.
- Notes and counts are non-printing and excluded from the raster composite.

## Edge cases

- **Extended-only Count**: on a Standard-parity build, the Count tool must be
  absent/disabled; Note remains available.
- **Print/export**: notes and counts never render into raster output; verify
  they are excluded from merge/flatten and Save For Web.
- **Format support**: notes survive PSD, PDF, TIFF but not all formats (e.g.
  JPEG/PNG) — warn or drop per format.
- **Many notes/counts**: overlay rendering must cull off-screen markers; label
  size 72 with dense counts can overlap — acceptable.
- **Count group deletion**: the last group cannot be deleted while counts
  exist, or deletion clears its counts; CS6 behavior unverified.
- **Auto-count**: touching/anti-aliased components may merge or split;
  result is selection-definition dependent.
- **Measurement Log**: recording does not mutate on-image counts; clearing
  image counts leaves Log rows.
- **32-bit/CMYK/Lab**: notes/counts are color-model independent overlays.
- **GPU-unavailable**: overlays render on CPU.
- **Undo/redo**: deleting a group must restore its marks on undo.

## Parity acceptance criteria

1. Given the Note tool with Author "A" and a color, clicking places a note icon
   and opens the Notes panel; typing text and saving as PSD, then reopening,
   restores the note, author, and color.
2. Given `View > Show > Notes` off, note icons are hidden; on, they reappear.
3. Given a printed or merged output, note icons do not appear in the raster.
4. Given the Count tool (Extended), clicking N times yields count labels 1..N
   in the active group.
5. Given two count groups, counts added to group B increment only B; hiding B
   hides only B's markers.
6. Given `Alt`/`Option`-click on a marker, it is removed and the total
   decrements; given Clear, the active group resets to 0.
7. Given `File > Save` then reopen (PSD), all count groups, markers, and numbers
   are restored.
8. Given `Analysis > Record Measurements` with the Count data point enabled,
   one Log row per record is appended, and a later Clear leaves them unchanged.
9. Given a Magic Wand selection of K disjoint regions, automatic counting
   reports K.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  "Navigation, notes, and measuring tools gallery" (Note tool description,
  Count "(Photoshop Extended only)"); "Counting objects in an image (Photoshop
  Extended)" (manual count, count groups, color, marker size 1–10, label size
  8–72, move/delete/clear, show/hide, record to Measurement Log, save with
  file, automatic counting via Magic Wand/Color Range); "Measurement
  (Photoshop Extended)" and Measurement Log data points (Count); "Crop and
  straighten" cross-reference for the note tool grouping; key shortcut table
  (`I` group; Count tool marked `*`).
- `https://www.bapugraphics.com/blog/adobe-photoshop-note-tool` — Note tool
  procedure: options-bar Author and Color, click to place a note, Notes panel
  text entry, non-printing behavior, `View > Show > Notes` / `View > Extras`,
  double-click to edit, `Window > Notes` to cycle notes; notes save in PSD, PDF,
  or TIFF (derived from older Adobe documentation; use as secondary).
- `https://www.photoshopforphotographers.com/3101-1901/Help_guide/tp/Notes_palette.html`
  — states that documents can be saved in the Photoshop (PSD), PDF, or TIFF
  formats, and describes the Notes panel (search snippet only; page not directly
  fetched).

## Open questions

- **Audio Annotation CS6 status**: is it still present alongside the Note tool?
  *Resolves with:* CS6 UI capture.
- **Note serialization keys** in PSD/PDF/TIFF and whether notes are multiple or
  a single annotation block. *Resolves with:* file-format specs plus a
  CS6-saved sample.
- **Count default marker/label sizes and the exact default group name**.
  *Resolves with:* CS6 observation.
- **Count group deletion semantics** when marks exist (blocked vs. clear).
  *Resolves with:* CS6 observation.
- **Exact Measurement Log record schema** for the Count data point (columns and
  labels). *Resolves with:* CS6 export/observation, coordinated with
  `10-workflow-io/measurement-and-count.md`.
- **Auto-count connected-component rules** (8- vs 4-connectivity, minimum size,
  handling of anti-aliasing). *Resolves with:* controlled test selections on
  CS6.
- **Whether Note/Count are recorded as action steps** and their serialized
  parameters. *Resolves with:* CS6 action/scripting docs.
