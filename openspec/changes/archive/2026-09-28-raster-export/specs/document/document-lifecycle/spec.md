## MODIFIED Requirements

### Requirement: Save and Save As

The system SHALL write the active document in its current output format, chosen
by the save path's extension, writing atomically so a failed write never
truncates an existing file. A `psd` path SHALL write the document's composite
and layer tree through the PSD codec; a `psb` path SHALL force a version-2 PSB
container through the codec. A supported raster path (PNG, JPEG, TIFF, WebP, or
BMP) SHALL write the flattened composite through the Qt-boundary raster encode
edge. Save As SHALL default its format to PSD unless the document is a single
pixel or background layer imported from a supported raster, in which case it
SHALL default to that source format. When the document's recorded path is a
raster format and the document has more than one layer, a group, an adjustment,
or a type layer, Save SHALL behave as Save As rather than silently flattening
into that raster file. Saving a document that has more than one layer, a group,
an adjustment, or a type layer to a format that cannot hold them SHALL warn
before writing and then write the flattened composite. The `composite`
serialized to PSD SHALL be the active document's current rendered composite, so
a save after any edit round-trips to the edited pixels rather than to a pre-edit
merged image. The `write_psd` byte layout and its use of `doc.composite.channels`
to derive the PSD colour-plane count MUST NOT change; an RGB document's merged
composite is RGBA (four planes) after a canvas rebuild, while a grayscale
document keeps its composite colour-plane count. Save on an untitled document
SHALL behave as Save As. A successful save SHALL record the file path and clear
the modified state.

#### Scenario: Save As writes a file

- **WHEN** the user runs Save As and chooses a path
- **THEN** a valid file of the chosen path's format is written at that path and the document is no longer modified

#### Scenario: Save uses the recorded path

- **WHEN** a document has a path and the user runs Save
- **THEN** the file at that path is overwritten atomically, in that path's format

#### Scenario: Save on an untitled document prompts for a path

- **WHEN** Save is invoked on an untitled document
- **THEN** a Save As path is requested before any write

#### Scenario: Save after an edit writes the edited composite

- **WHEN** an edit changes the canvas and the document is saved and read back
- **THEN** the read-back document's composite equals the edited rendered composite, not the pre-edit merged image, and its colour-plane count matches the saved document's colour-plane count

#### Scenario: Save As defaults to the import source format

- **WHEN** a single-layer PNG imported through `File > Open` is saved with Save As
- **THEN** the Save As dialog preselects PNG as the format

#### Scenario: A raster save clears modified and re-saves the same format

- **WHEN** a document is saved to `photo.png` and then edited and saved again
- **THEN** the second save writes a PNG to `photo.png` and clears the modified state

#### Scenario: A layered document saved to a flat format warns and flattens

- **WHEN** the user saves a document with more than one layer to a PNG
- **THEN** a warning is shown that the format cannot hold the document's features, and on confirmation the flattened composite is written as a PNG

#### Scenario: A layered document defaults Save As to PSD

- **WHEN** a PNG imported through `File > Open` has a layer or an adjustment added and Save As is invoked
- **THEN** the Save As dialog preselects PSD, not PNG

#### Scenario: Save on a layered raster path behaves as Save As

- **WHEN** a layered document recorded at a `.png` path is saved
- **THEN** the format-aware Save As dialog opens (defaulting to PSD) instead of overwriting the PNG with a flattened composite

#### Scenario: A PSB path writes a PSB container

- **WHEN** a document is saved to a `.psb` path
- **THEN** the written file is a version-2 PSB

## ADDED Requirements

### Requirement: File dialog format filters

The Open, Save, Place, and Open As Smart Object file dialogs SHALL share one set
of format filters: one entry per format, named for the format and listing its
extensions in uppercase, with the patterns separated by spaces. The Open dialog
SHALL offer an `All Formats` entry listing every supported import format and
SHALL select it by default; the Save As type list SHALL also offer `All Formats`,
which names no single format and therefore falls back to the document's smart
default when the typed name has no extension. The Export As dialog SHALL present
its format via its own combo and SHALL filter the file dialog with a single
uppercase entry for the chosen format; the path's extension remains the
authoritative format. The "File type" combo
SHALL be non-editable, so it cannot be cleared, and SHALL support rolling
type-ahead: a printable key selects the first filter whose name starts with that
character (case-insensitively) and repeating the same key within the
keyboard-input interval advances to the next matching filter, wrapping. The
format written to disk SHALL use a lowercase extension regardless of the
filter's case.

#### Scenario: Save As lists one filter per format

- **WHEN** the Save As dialog is opened
- **THEN** its type list contains a separate uppercase entry for each of Photoshop, Photoshop Large Format, PNG, JPEG, TIFF, WebP, and BMP

#### Scenario: Open selects All Formats

- **WHEN** the Open dialog is opened
- **THEN** its type list contains an `All Formats` entry and that entry is selected

#### Scenario: Open As Smart Object uses the shared filters

- **WHEN** the Open As Smart Object dialog is opened
- **THEN** its type list matches the Open dialog's, including an `All Formats` entry and the raster formats

#### Scenario: The type combo rolls with a repeated key

- **WHEN** the "File type" combo has focus and the user presses a letter, then the same letter again within the keyboard-input interval
- **THEN** the selection moves to the next filter whose name starts with that letter, wrapping at the end

#### Scenario: The type combo cannot be cleared

- **WHEN** the "File type" combo is edited by keyboard
- **THEN** it selects a filter and never becomes empty

#### Scenario: Extensions are written lowercase

- **WHEN** a document is saved with a filter whose pattern is uppercase and the typed name has no extension
- **THEN** the appended extension is lowercase (for example `.png`)

### Requirement: Open As Smart Object accepts raster sources

`File > Open As Smart Object` SHALL accept a PSD/PSB source as-is and SHALL accept
a supported raster source (PNG, JPEG, GIF, BMP, TIFF, WebP) by embedding its
decoded layer as a smart object. The resulting document SHALL have a single
smart-object layer whose contents can be edited; a file that is neither PSD/PSB
nor a supported raster SHALL be refused without changing the view.

#### Scenario: A raster opens as a single smart-object layer

- **WHEN** File > Open As Smart Object is used on a PNG
- **THEN** the document has a single smart-object layer whose contents can be edited

#### Scenario: An unsupported source is refused

- **WHEN** File > Open As Smart Object is used on a file that is neither PSD/PSB nor a supported raster
- **THEN** no document is created

### Requirement: File dialog places

The Open, Save, Place, Open As Smart Object, and Export As file dialogs SHALL
expose a Places sidebar containing the filesystem root, the mounted volumes, the
XDG user folders, the desktop bookmark directories (KDE `user-places.xbel` and
GTK `gtk-3.0/bookmarks`), and the application's recent locations. Only existing
directories SHALL be offered, and the list SHALL be deduplicated by canonical
path. When the application runs inside a sandbox (Flatpak or Snap), the dialogs
SHALL instead delegate to the platform/portal file chooser, which supplies the
host's places.

#### Scenario: Root and home are offered

- **WHEN** any file dialog is opened
- **THEN** its Places sidebar contains the filesystem root and the user's home folder

#### Scenario: Mounted volumes and bookmarks are offered

- **WHEN** the system has a mounted volume or a KDE/GTK bookmark pointing at a directory
- **THEN** that directory appears in the sidebar

#### Scenario: Missing places are omitted

- **WHEN** a bookmark points at a path that no longer exists or is not a directory
- **THEN** that entry is left out of the sidebar

#### Scenario: Recent locations are offered

- **WHEN** a document has been saved through the application
- **THEN** its parent directory appears among the recent locations in the sidebar

#### Scenario: A sandboxed run delegates to the platform chooser

- **WHEN** the application runs inside a Flatpak or Snap sandbox
- **THEN** the file dialogs use the platform/portal chooser rather than the built-in dialog
