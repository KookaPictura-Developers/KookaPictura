## MODIFIED Requirements

### Requirement: Tool status and cursor hints

The system SHALL show a context hint for the active tool in the bottom status bar
as a hint bar of bordered keycaps with descriptions, and SHALL set a
tool-appropriate cursor while the tool is active. For Brush and Pencil the cursor
SHALL be blank so the drawn brush-size ring is the pointer affordance. When the
active tool would edit pixels and the target layer is pixel-locked, the cursor
SHALL indicate that the action is not allowed, and a refusal SHALL be reported to
the user rather than the action silently doing nothing. **The canvas SHALL also
show a Block/Forbidden cursor when a pixel-editing tool's active layer is
invisible (an edit would be refused), and the transient Alt eyedropper SHALL
show the eyedropper cursor while Alt is held with a paint tool active. The
cursor SHALL be resolved in one place with the precedence transient eyedropper >
transform session > blank paint > invisible target > pixel-locked target >
tool default, so the branches cannot disagree.**

#### Scenario: Status hint reflects the tool

- **WHEN** a tool becomes active
- **THEN** the status-bar hint bar shows that tool's context keycaps

#### Scenario: Brush uses a blank cursor [ltf_blank_cursor]

- **WHEN** Brush or Pencil is active over the canvas
- **THEN** the canvas cursor is blank and the brush-size ring is drawn

#### Scenario: A locked target marks the cursor [ltf_locked_cursor]

- **WHEN** a pixel-editing tool is active over a pixel-locked layer
- **THEN** the canvas cursor indicates the action is not allowed and an attempted
  edit reports a refusal

#### Scenario: An invisible target marks the cursor [ltf_invisible_cursor]

- **WHEN** a pixel-editing tool is active over a layer whose visibility is off
- **THEN** the canvas shows a Block/Forbidden cursor and an attempted paint is
  refused

#### Scenario: The transient eyedropper cursor shows [ltf_alt_cursor]

- **WHEN** Alt is held with a paint tool active over the canvas
- **THEN** the canvas shows the eyedropper cursor, which takes precedence over
  the blank paint cursor

## ADDED Requirements

### Requirement: Transient alt-tool cursor is resolved centrally

The cursor precedence SHALL live in one `refreshCursor` path so the transient
eyedropper, the invisible-layer refusal, and the existing blank/locked cursors
are not re-evaluated in separate handlers. Changing the active layer, its
visibility, or the held modifiers SHALL refresh the cursor through that one path
without a per-tool lookup.

#### Scenario: One path resolves all cursor states [ltf_cursor_precedence]

- **WHEN** the active layer becomes invisible, then visible again, with a paint
  tool active
- **THEN** the canvas cursor changes through the single precedence path each time
  and no other handler re-sets it
