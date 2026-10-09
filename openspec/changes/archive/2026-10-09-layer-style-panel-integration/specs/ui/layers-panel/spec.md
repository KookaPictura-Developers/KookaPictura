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
The **Layer Style** (`fx`) button SHALL be active: a click SHALL open the Layer
Style dialog on Blending Options for the active layer when that layer can carry
a style, and an `Alt`-click SHALL toggle all effects through the layer-style
bridge in one undoable step. The **Link Layers** button SHALL be shown disabled
until its operation lands.

#### Scenario: The implemented strip buttons carry icons

- **WHEN** the Layers panel is shown
- **THEN** its mask, fx, fill/adjustment, group, new-layer, and delete buttons each carry their documented icon

#### Scenario: A deferred button is disabled

- **WHEN** the Layers panel is shown before the link operation exists
- **THEN** the link button is disabled

#### Scenario: The mask button adds a mask [lmk_strip]

- **WHEN** the user clicks the Add Layer Mask strip button with no selection
- **THEN** a reveal-all mask is added to the active layer in one undoable step

#### Scenario: The mask button respects a selection [lmk_strip_sel]

- **WHEN** the user clicks the Add Layer Mask strip button while a selection exists
- **THEN** a reveal-selection mask is added to the active layer in one undoable step

#### Scenario: Alt-clicking the mask button hides all [lmk_strip_alt]

- **WHEN** the user `Alt`-clicks the Add Layer Mask strip button
- **THEN** a hide-all mask is added to the active layer in one undoable step

#### Scenario: The fx button opens Blending Options

- **WHEN** the user clicks the Layer Style strip button with an active layer that can carry a style
- **THEN** the Layer Style dialog opens on Blending Options for that layer

#### Scenario: Alt-clicking the fx button toggles all effects

- **WHEN** the user `Alt`-clicks the Layer Style strip button
- **THEN** every layer's effects are shown when any were hidden and hidden when any were shown, as one undoable step

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, a
clipping-mask indicator for a clipped layer, the clipping indentation and base
underline, the layer-mask thumbnail when a mask is present, an adjustment badge
when adjustment content is present, and a layer-style `fx` badge when the layer
carries a layer style (an `lfx2` or legacy `lrFX` block). A layer whose lock
state has any flag set SHALL also show a lock badge at the right side of its
row; an unlocked layer SHALL show none. The visibility toggle SHALL be an eye
icon (`layers.eyeOn`/`layers.eyeOff`) drawn slightly inset from the panel's left
edge and at the same x for every row, independent of nesting depth; the nesting
indentation SHALL apply to the thumbnail and name, not to the visibility toggle.
A layer whose color label is not `None` SHALL tint the **visibility toggle's own
background** (`eyeRect`) with that label color behind the eye glyph, and SHALL
additionally paint a small color-label **chip** — a short bar of the label
color, one row tall — at the start of the row content immediately before the
thumbnail, with the thumbnail and name laid out after the chip so no rect
overlaps; the tint SHALL keep the eye glyph and any selection highlight legible,
and the label color elsewhere on the row SHALL fall back to the row background.
The chip is a paint-only affordance and SHALL NOT alter hit-testing beyond the
thumbnail and name rects accounting for it. A group with at least one child
SHALL show a disclosure icon — right when collapsed, down when expanded — at its
indented position, and clicking that icon SHALL expand or collapse the group. A
**regular (non-group) layer's thumbnail SHALL be drawn over a cached two-tone
checkerboard** so transparency reads under it, while a group keeps its folder
glyph and no checkerboard. **Every thumbnail SHALL carry a 1 px black outline**,
and **when exactly one layer is active** (the same singular active-layer
resolution tool edits use) its thumbnail SHALL additionally show white 1 px
corner brackets drawn one pixel outside the outline; with zero or multiple active
layers no brackets are drawn. A layer whose row reports the smart-object
projection SHALL paint the `layers.kindSmartObject` badge on the thumbnail's
lower-right corner, whether the smart object is embedded or placed/external; the
badge SHALL be omitted when the asset is missing. Row typography SHALL derive
from the layer: a `background` layer's name SHALL be italic/cursive, every other
name normal, and a layer that is a member of the frame's link set or a
placed/external smart object SHALL be underlined through the new
`LayerRowLinkedRole` and `LayerRowPlacedRole` projections. The delegate SHALL
report a row height of at least **28 px** through one named constant used by both
`sizeHint` and the vertical centring math. If an expected icon asset is
unavailable, the delegate SHALL omit that badge while keeping the row legible
rather than fail.

#### Scenario: The visibility toggle is an eye icon [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its visibility toggle is drawn from the eye icon asset, and a hidden
  layer uses the off variant

#### Scenario: The eye is left-anchored for every depth [lpr_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its eye icon is at the same x as a top-level row's, while its
  thumbnail and name are indented

#### Scenario: The color label tints only the eye toggle [lpr_label_tint]

- **WHEN** a layer with a non-`None` color label is shown
- **THEN** the eye toggle's background is filled with that label color behind
  the eye glyph and a chip of the same color is painted at the row content edge
  before the thumbnail, and the rest of the row background is unchanged

#### Scenario: The label chip sits before the thumbnail [lpr_label_chip]

- **WHEN** a labeled row is shown
- **THEN** the chip is drawn between the disclosure/clipping glyph and the
  thumbnail, and the thumbnail's left edge follows the chip so the two do not
  overlap

#### Scenario: An unlabeled layer has no gutter tint [lpr_label_tint_none]

- **WHEN** a layer with color label `None` is shown
- **THEN** its eye gutter uses the normal row background and no chip is painted

#### Scenario: A smart object shows the badge [lpr_smart_badge]

- **WHEN** an embedded or placed smart-object layer is shown
- **THEN** its thumbnail draws the smart-object badge on the lower-right corner

#### Scenario: A non-smart layer shows no smart badge [lpr_smart_badge_absent]

- **WHEN** an ordinary pixel layer is shown
- **THEN** no smart-object badge is drawn on its thumbnail

#### Scenario: A group shows a disclosure icon [lpr_chevron]

- **WHEN** a group with children is shown collapsed and then expanded
- **THEN** it shows the right-pointing icon when collapsed and the down-pointing
  icon when expanded, and clicking the icon toggles it

#### Scenario: A masked layer shows a mask thumbnail [m39_badges]

- **WHEN** a layer carries a layer mask
- **THEN** its row shows a mask thumbnail and the mask-presence flag is set

#### Scenario: A clipped layer shows the clipping indicator [lpc_clip]

- **WHEN** a layer is clipped to the layer below it
- **THEN** its row draws the clipping-mask indicator in addition to the
  indentation and the base layer's underline

#### Scenario: A group row is expandable [m39_badges]

- **WHEN** a group has children
- **THEN** its row is marked expandable and shows an expand/collapse control

#### Scenario: A locked layer shows a right-side lock badge [lpc_lockbadge]

- **WHEN** a layer has any lock flag set
- **THEN** its row draws a lock badge on the right side, and an unlocked layer
  draws no lock badge

#### Scenario: A regular thumbnail shows a checkerboard [lpr_thumb_checker]

- **WHEN** a regular (non-group) layer with translucent pixels is shown
- **THEN** its thumbnail is drawn over a two-tone checkerboard, while a group
  row keeps its folder glyph and draws no checkerboard

#### Scenario: Thumbnails are outlined and the active layer is bracketed [lpr_thumb_bracket]

- **WHEN** a regular layer row is shown with a 1 px black thumbnail outline
- **THEN** the singular active layer's thumbnail additionally shows white
  corner brackets one pixel outside the outline, and a multi- or zero-selection
  row shows no brackets

#### Scenario: Row typography follows the layer kind [lpr_row_fonts]

- **WHEN** the rows for the Background, a linked layer, and a placed smart
  object are inspected
- **THEN** the Background name is italic/cursive, ordinary names are normal, and
  the linked and placed rows are underlined through their roles

#### Scenario: The row height has a floor [lpr_row_height]

- **WHEN** a row's size hint is read
- **THEN** its height is at least 28 px from the single named row-height
  constant

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

#### Scenario: A styled layer shows the fx badge

- **WHEN** a layer that carries a layer style is shown
- **THEN** its row draws the `layers.fx` badge at its right edge beside the lock
  and mask badges, and a layer with no style draws no fx badge

## ADDED Requirements

### Requirement: Layer style panel integration

The Layers panel SHALL be the surface that reaches the layer-style feature. A
row double-click outside the name and the eye/mask controls SHALL open the Layer
Style dialog on Blending Options for that layer when the layer can carry a style
(`layer_style_can_edit`), and SHALL do nothing otherwise. The row context menu
SHALL offer **Blending Options…**, **Copy Layer Style**, **Paste Layer Style**,
and **Clear Layer Style** as implemented commands: Blending Options SHALL appear
for pixel, background, group, and type rows and SHALL be enabled only where the
layer can carry a style; Copy and Clear SHALL be enabled when the row's layer
carries a style; Paste SHALL be enabled when the app-wide style clipboard is
non-empty. Choosing a command SHALL act on the row's layer through the
layer-style bridge and SHALL record through the bridge's own history.

An `Alt`-click on a styled-or-adjustment row's right-edge `fx` region SHALL
toggle every layer's effects through `layer_style_set_all_visible`, showing
them when all were hidden and hiding them when any were shown, as one undoable
step.

The filter row's **Effect** dimension SHALL be enabled and SHALL list the ten
effect keys from `layer_style_effect_names()`; selecting an effect SHALL match
the rows whose layer carries that effect (and, as with the other dimensions,
their ancestors), and a row whose layer does not carry it SHALL be filtered out.

#### Scenario: A double-click opens Blending Options

- **WHEN** a pixel layer row is double-clicked outside its name and its eye/mask controls
- **THEN** the Layer Style dialog opens on Blending Options for that layer

#### Scenario: A background double-click does not open a style

- **WHEN** the Background row is double-clicked outside its name
- **THEN** the Background conversion path runs and no Layer Style dialog opens

#### Scenario: The style rows report their enablement

- **WHEN** the row menu is opened for a pixel layer that carries a color overlay and has a copied style available
- **THEN** Blending Options, Copy Layer Style, Paste Layer Style, and Clear Layer Style are all enabled

#### Scenario: An unstyled pixel row disables Copy and Clear

- **WHEN** the row menu is opened for a pixel layer with no layer style
- **THEN** Blending Options is enabled and Copy Layer Style, Paste Layer Style, and Clear Layer Style are disabled

#### Scenario: An adjustment row omits the style rows

- **WHEN** the row menu is opened for an adjustment layer
- **THEN** it does not offer Blending Options, Copy Layer Style, Paste Layer Style, or Clear Layer Style

#### Scenario: The row menu clears a row's style

- **WHEN** Clear Layer Style is chosen for a pixel row that carries a style
- **THEN** the row's style is removed in one undoable step and its fx badge disappears

#### Scenario: Alt-clicking the fx region toggles all effects

- **WHEN** the user `Alt`-clicks the fx badge of a styled row while effects are shown
- **THEN** every layer's effects are hidden in one undoable step, and a second `Alt`-click shows them again

#### Scenario: The Effect dimension matches styled rows

- **WHEN** the filter uses the Effect dimension with an effect a layer carries
- **THEN** that layer's row is shown and rows without the effect are hidden

#### Scenario: The Effect dimension lists the effect keys

- **WHEN** the filter's Effect dimension is selected
- **THEN** its choices are the ten effect keys from the layer-style effect list
