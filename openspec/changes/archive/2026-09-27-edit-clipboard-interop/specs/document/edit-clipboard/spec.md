# edit-clipboard Specification

## ADDED Requirements

### Requirement: Clip import and export

`Clip::from_rgba(width, height, rgba)` SHALL build a clip from packed straight
RGBA at the canvas origin with full coverage, and SHALL return `None` for a zero
dimension or a buffer whose length is not `width * height * 4`.
`Clip::masked_rgba()` SHALL return the clip's packed straight RGBA with each
alpha scaled by its selection coverage (`round(alpha * coverage / 255)`), and a
paste SHALL show exactly those pixels.

#### Scenario: A foreign image is a fully covered clip at the origin

- **WHEN** `Clip::from_rgba(2, 1, rgba)` is called with 8 bytes
- **THEN** the clip's rect is `(0, 0)–(2, 1)`, its mask is fully covered, and `masked_rgba` returns `rgba` unchanged

#### Scenario: A malformed image refuses

- **WHEN** `Clip::from_rgba` is called with a short buffer or a zero dimension
- **THEN** it returns `None`

#### Scenario: Coverage folds into alpha

- **WHEN** a clip with alpha 255 and 200 and coverage 128 and 0 is exported
- **THEN** `masked_rgba` carries alpha 128 and 0 with the colors unchanged

### Requirement: System clipboard interop

After a successful Copy, Cut, or Copy Merged the shell SHALL put the clip's
`masked_rgba` on the system clipboard as an image. Before any paste, when another
application has replaced the system clipboard since our last write and it holds
an image, that image SHALL replace the bridge copy (at the canvas origin);
otherwise the bridge copy SHALL be pasted. Paste and Paste in Place SHALL be
enabled with a document when either the bridge copy or a system-clipboard image
exists (Paste Into / Paste Outside additionally need a selection). Purge ▸
Clipboard SHALL also clear the system clipboard while it holds our own export,
and SHALL NOT clear another application's clipboard.

#### Scenario: Copy exports the selection

- **WHEN** `edit_clipboard` copies a 3×3 selection through the Copy command
- **THEN** the system clipboard holds a 3×3 image

#### Scenario: Another application's image pastes

- **WHEN** another application puts a green 2×2 image on the system clipboard and Paste in Place runs
- **THEN** one "Paste" state adds a layer and the composite at `(0, 0)` is green

#### Scenario: Purge clears our export

- **WHEN** Copy Merged exports the clip and Purge ▸ Clipboard runs
- **THEN** the bridge clipboard is empty, the system clipboard holds no image, and Paste is disabled

### Requirement: Paste in Place

`Edit ▸ Paste Special ▸ Paste in Place` SHALL add the clip as a new layer with
its top-left at the clip's own document position (the canvas origin for an
imported image), by the same placement, current-layer, and one-"Paste"-state
rules as Paste, and SHALL ship without a default shortcut.

#### Scenario: Cut then Paste in Place restores the pixels

- **WHEN** `edit_clipboard` cuts a 3×3 selection at `(2, 2)` and runs Paste in Place
- **THEN** one "Paste" state adds a layer and the composite inside the cut region shows the original pixels again
