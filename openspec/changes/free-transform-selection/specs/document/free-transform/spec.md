## ADDED Requirements

### Requirement: Free Transform of selected pixels

When a non-empty, document-sized selection is active and the target is a pixel layer or the Background that is not pixel-locked, not transparency-locked while it has alpha, and not position-locked unless it is the Background, beginning a Free Transform session (in any mode) SHALL lift the selected pixels: they are copied into a floating layer directly above the target, trimmed to the selection bounds, and cleared from the target as `Edit > Clear` clears them. The session SHALL then transform the floating layer. Committing a non-identity transform SHALL merge the floating layer back into the target, keeping the target's name, locks, and Background status, SHALL record exactly one `"Free Transform"` history state, and SHALL drop the selection. Cancelling, or committing an identity transform, SHALL restore the document exactly as it was before the lift and record nothing. `layer_can_free_transform` SHALL report such a target as transformable, so `Edit > Free Transform` is enabled for it. Without a selection, the existing refusals (including the Background) SHALL apply unchanged.

#### Scenario: A Background with no selection stays refused

- **WHEN** Ctrl+T is pressed on an opened photo whose only layer is the Background and no selection is active
- **THEN** `Edit > Free Transform` is disabled and no session begins

#### Scenario: A selection on the Background is lifted and transformed

- **WHEN** a rectangle is selected on the Background, Ctrl+T is pressed, the box is dragged 10 px right and 5 px down, and Return commits
- **THEN** one `"Free Transform"` state is recorded, the document still has a single Background layer, the vacated pixels are white, the moved pixels land at the offset, and the selection is gone

#### Scenario: Cancel restores the lifted pixels

- **WHEN** a session that lifted a selection is cancelled with Escape
- **THEN** the floating layer is gone, every pixel is as before, and no history state is recorded

## MODIFIED Requirements

### Requirement: Skew, Distort, and Perspective menu commands

The application SHALL provide implemented commands `Edit > Transform > Skew`,
`Edit > Transform > Distort`, and `Edit > Transform > Perspective` that begin a
Free Transform session in the matching mode on the active layer, using the same
active-layer resolution and refusal as `Edit > Free Transform`. Each command
SHALL be enabled exactly when `Edit > Free Transform` is enabled, including for
a target whose selected pixels Free Transform lifts.

#### Scenario: The Distort command enters a Distort session

- **WHEN** `Edit > Transform > Distort` is invoked with exactly one transformable layer active
- **THEN** a session begins in `Distort` mode on that layer

#### Scenario: A non-transformable target leaves the command disabled

- **WHEN** the active layer is a group, an adjustment layer, or position-locked, or is the Background with no active selection
- **THEN** the Distort/Skew/Perspective commands are disabled and invoking them begins no session
