## ADDED Requirements

### Requirement: Canvas cursor refreshes on an active-layer change

When the active layer's visibility or identity changes, the canvas cursor SHALL
be refreshed without requiring a pointer move, so an invisible active layer shows
the Block/Forbidden cursor (and a visible one restores the tool cursor)
immediately after an eye toggle or a layer selection. The cursor resolution
itself SHALL keep the existing precedence: a transient Alt eyedropper wins, then
the invisible/locked refusal, then the tool cursor.

#### Scenario: Hiding the active layer shows the Block cursor [tf_visible_cursor_refresh]

- **WHEN** the active layer is made invisible while a pixel-editing tool is
  active and the pointer is over the canvas
- **THEN** the canvas cursor becomes the Block/Forbidden cursor without moving
  the pointer

#### Scenario: Showing the active layer restores the tool cursor [tf_visible_cursor_restore]

- **WHEN** the active layer is made visible again
- **THEN** the canvas cursor returns to the active tool's cursor without moving
  the pointer
