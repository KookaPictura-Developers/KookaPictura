# ui/character-paragraph-panels Specification

## Purpose
Provides the Character and Paragraph panels as live attribute editors over the type model, and wires them into the Window and Type menus so they can be shown and hidden like the other panels.

## Requirements

### Requirement: Character panel edits the active type layer

The Character panel SHALL show the active type layer's character attributes —
font family, size, leading, kerning, tracking, horizontal and vertical scale,
baseline shift, anti-aliasing method, fill colour, and the caps, super/subscript,
underline, and strikethrough toggles — and SHALL apply an edit to that layer
through the type bridge, re-rendering it and recording exactly one history state.
When no type layer is active it SHALL present the model defaults and SHALL NOT
record a state.

#### Scenario: Panel reflects the active type layer

- **WHEN** a type layer is selected
- **THEN** the Character panel shows that layer's size, leading, tracking, scale, anti-aliasing, colour, and toggles

#### Scenario: Editing an attribute updates the layer as one state

- **WHEN** the size field is committed on the active type layer and then undone once
- **THEN** the layer's rendered text reflects the new size and the single undo restores the previous size

#### Scenario: No type layer shows the defaults

- **WHEN** no type layer is active
- **THEN** the panel shows the default character attributes and committing a value records no history state

### Requirement: Paragraph panel edits the active type layer

The Paragraph panel SHALL show the active type layer's paragraph attributes —
alignment and justification, left/right/first-line indents, space before and
after, hanging punctuation, hyphenation, and composer — and SHALL apply an edit
to that layer through the type bridge, re-rendering it and recording exactly one
history state.

#### Scenario: Panel reflects the active type layer

- **WHEN** a type layer is selected
- **THEN** the Paragraph panel shows its alignment, indents, paragraph spacing, hanging punctuation, hyphenation, and composer

#### Scenario: Editing a paragraph attribute updates the layer

- **WHEN** the first-line indent is committed and then undone once
- **THEN** the layer's stored paragraph attributes reflect the change and the single undo restores the previous value

### Requirement: Window and Type menus toggle the panels

`Window > Panels > Character`/`Paragraph` and `Type > Panels > Character`/
`Paragraph` SHALL be enabled checkable commands. Checking one SHALL show the
panel in the panel column, unchecking SHALL hide it, and the checked state SHALL
reflect the panel's actual visibility.

#### Scenario: Toggling shows and hides the panel

- **WHEN** `Window > Panels > Character` is checked and then unchecked
- **THEN** the Character panel becomes visible and then hidden, and the menu item's checked state follows

#### Scenario: Menu state reflects visibility

- **WHEN** a panel is shown or hidden by any means
- **THEN** its Window and Type menu entries report the matching checked state

### Requirement: The type options bar opens the panels

The type options bar's Panel button SHALL toggle the Character and Paragraph
panels, matching CS6.

#### Scenario: Options-bar button toggles the type panels

- **WHEN** the type options bar's Panel button is pressed
- **THEN** the Character and Paragraph panels toggle visible or hidden
