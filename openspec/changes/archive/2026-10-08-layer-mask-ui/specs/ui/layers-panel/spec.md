## MODIFIED Requirements

### Requirement: Layers panel action strip icons

The system SHALL give each Layers-panel action-strip button an icon from the
frozen layers asset set: `layers.link` (Link Layers), `layers.fx` (Layer Style),
`layers.mask` (Add Layer Mask), `layers.fillAdjustment` (New Fill / Adjustment
Layer), `layers.group` (New Group), `layers.newLayer` (New Layer), and
`layers.delete` (Delete). The buttons SHALL keep their text labels, and adding
an icon SHALL NOT change what each button does. The **Add Layer Mask** button
SHALL be active: a click SHALL add a `reveal-selection` mask when a selection
exists and a `reveal-all` mask otherwise, and an `Alt`-click SHALL add a
`hide-all` mask, each through the layer-mask bridge and each one undoable step.
Buttons whose operation is not yet implemented (link, fx) SHALL be shown
disabled until their operation lands.

#### Scenario: The implemented strip buttons carry icons

- **WHEN** the Layers panel is shown
- **THEN** its mask, fill/adjustment, group, new-layer, and delete buttons each carry their documented icon

#### Scenario: A deferred button is disabled

- **WHEN** the Layers panel is shown before the link or fx operation exists
- **THEN** that button is disabled

#### Scenario: The mask button adds a mask [lmk_strip]

- **WHEN** the user clicks the Add Layer Mask strip button with no selection
- **THEN** a reveal-all mask is added to the active layer in one undoable step

#### Scenario: The mask button respects a selection [lmk_strip_sel]

- **WHEN** the user clicks the Add Layer Mask strip button while a selection exists
- **THEN** a reveal-selection mask is added to the active layer in one undoable step

#### Scenario: Alt-clicking the mask button hides all [lmk_strip_alt]

- **WHEN** the user `Alt`-clicks the Add Layer Mask strip button
- **THEN** a hide-all mask is added to the active layer in one undoable step

### Requirement: Panel and row menus

The system SHALL provide a panel menu and a row context menu. The panel menu
SHALL offer `Panel Options…`, New Layer, New Group, Duplicate Layer(s), Delete
Layer(s), Group Layers, Ungroup Layers, Move Layer Up, and Move Layer Down;
commands the system does not yet implement SHALL appear disabled with a
`— not implemented yet` tooltip rather than being hidden.

The row context menu SHALL be assembled **per layer kind**. Each kind (pixel,
background, group, adjustment, type, shape, smart object) SHALL be offered the
commands that apply to it, and SHALL NOT be offered commands that do not apply
(for example, a group or adjustment row SHALL NOT offer `Export As…` or `Quick
Export as PNG`, and a group row SHALL NOT offer `Fill`-related content). Every
applicable command that is not yet implemented SHALL still appear, rendered
disabled, carrying a `— not implemented yet` tooltip, so that implementing a
command enables an existing row rather than adding one. The commands common to
every kind SHALL include Rename and a color-label submenu containing `None`,
`Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, and `Gray`.

For a pixel row the menu SHALL offer **Add Layer Mask**, **Delete Layer Mask**,
**Enable Layer Mask**, and **Disable Layer Mask**, each wired to the layer-mask
bridge: Add Layer Mask SHALL add a `reveal-selection` mask when a selection
exists and a `reveal-all` mask otherwise; Delete Layer Mask SHALL remove the
mask; and Enable/Disable Layer Mask SHALL set the mask's enabled state. Each
applied action SHALL be one undoable step.

Right-clicking the visibility toggle SHALL offer show/hide this layer only and
show/hide all. The bottom action strip SHALL remain exactly the CS6 seven
buttons, so Move Up/Down SHALL NOT appear in the strip.

#### Scenario: The panel menu contains the wired commands [m39_menus]

- **WHEN** the panel menu is opened
- **THEN** it contains Panel Options, New Layer, New Group, Duplicate, Delete,
  Group, Ungroup, Move Up, and Move Down; commands that are not yet implemented
  are shown disabled with the `— not implemented yet` tooltip

#### Scenario: The row menu sets a color label [m39_menus]

- **WHEN** the user right-clicks a row and picks Red from the color-label submenu
- **THEN** that layer's color label becomes Red

#### Scenario: Move Up/Down reorders from the menu [m39_menus]

- **WHEN** the user picks Move Layer Up for a row that is not the top of its container
- **THEN** the layer swaps with the layer above it within its container in one undo step

#### Scenario: The action strip has no Move buttons [m39_strip]

- **WHEN** the Layers panel is shown
- **THEN** its bottom strip contains exactly the seven CS6 buttons and no Move Up
  or Move Down button

#### Scenario: Pixel and smart-object rows offer export

- **WHEN** the user opens the row menu for a pixel layer or a smart-object layer
- **THEN** it contains Export As… and Quick Export as PNG

#### Scenario: Group and adjustment rows hide export

- **WHEN** the user opens the row menu for a group or an adjustment layer
- **THEN** it does not offer Export As… or Quick Export as PNG

#### Scenario: Type rows hide export

- **WHEN** the user opens the row menu for a type layer
- **THEN** it does not offer Export As… or Quick Export as PNG

#### Scenario: A pixel row offers its full command set

- **WHEN** the user opens the row menu for a pixel layer
- **THEN** it contains the common commands, the color-label submenu, `Export
  As…`, `Quick Export as PNG`, and the pixel-applicable mask, style, clipping,
  smart-object, and rasterize commands — each either enabled or disabled with a
  tooltip when not yet implemented

#### Scenario: The row menu adds and deletes a mask [lmk_rowmenu]

- **WHEN** the user picks Add Layer Mask for a pixel row, then Delete Layer Mask
- **THEN** a mask is added and then removed from the active layer, each in one undo step

#### Scenario: The row menu toggles mask enablement [lmk_rowmenu_enable]

- **WHEN** the user picks Enable Layer Mask or Disable Layer Mask for a masked layer
- **THEN** the mask's disabled bit is cleared or set in one undo step

#### Scenario: A group row omits content commands

- **WHEN** the user opens the row menu for a group row
- **THEN** it does not offer `Export As…`, `Quick Export as PNG`, or fill
  commands, and it offers `Ungroup Layers`

#### Scenario: A type row offers rasterize

- **WHEN** the user opens the row menu for a type layer
- **THEN** it offers `Rasterize Type`

#### Scenario: An unimplemented applicable command is disabled, not absent

- **WHEN** the user opens the row menu for a kind and a command applies to that
  kind but is not yet implemented
- **THEN** the command is present, disabled, and shows the `— not implemented
  yet` tooltip

## ADDED Requirements

### Requirement: Layer mask row indicators

When a layer row carries a layer mask, the delegate SHALL draw a link glyph
between the layer thumbnail and the mask thumbnail when that mask is linked to
its layer, and SHALL draw a red cross over the mask thumbnail when the mask is
disabled. Clicking the link glyph SHALL toggle the mask's linked state, and
`Shift`-clicking the mask thumbnail SHALL toggle the mask's enabled state,
through the layer-mask bridge; both clicks SHALL be consumed so they do not
start a rename, select another row, or begin a drag. The model SHALL expose the
per-row mask linked and disabled states through roles populated from the bridge.

#### Scenario: A linked mask shows the link glyph [lmk_row_link]

- **WHEN** a row whose mask is linked is shown
- **THEN** a link glyph is drawn between the layer and mask thumbnails

#### Scenario: A disabled mask shows a red cross [lmk_row_disabled]

- **WHEN** a row whose mask is disabled is shown
- **THEN** a red cross is drawn over the mask thumbnail

#### Scenario: Clicking the link glyph unlinks the mask [lmk_row_link_click]

- **WHEN** the user clicks the link glyph on a linked mask row
- **THEN** the mask is unlinked in one undoable step

#### Scenario: Shift-clicking the mask thumbnail disables it [lmk_row_shift]

- **WHEN** the user `Shift`-clicks the mask thumbnail of an enabled mask
- **THEN** the mask is disabled in one undoable step
