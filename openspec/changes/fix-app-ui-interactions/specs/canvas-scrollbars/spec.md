## ADDED Requirements

### Requirement: Scrollbars are always visible

Both the horizontal and the vertical scrollbar SHALL be visible at all times,
regardless of whether the document fits the viewport, so the canvas is freely
pannable. Each bar SHALL remain a pure projection of the canvas offset and zoom
through the shared range helper, and the canvas SHALL remain the single source of
truth. Hiding a bar by an as-needed policy is removed; the bars SHALL NOT collapse
to hidden when the document fits.

#### Scenario: Bars stay visible when the document fits [lcs_always_visible]

- **WHEN** the document is smaller than the viewport along both axes
- **THEN** both scrollbars are still shown and the canvas can still be panned
  within the shared range

#### Scenario: Bars are visible when the document overflows

- **WHEN** the canvas is zoomed so the document is larger than the viewport along
  an axis
- **THEN** both scrollbars are shown with that axis's pan range

## REMOVED Requirements

### Requirement: Scrollbars hide when the document fits

**Reason**: The canvas is freely pannable within the shared reveal margin, so a
bar that hides whenever the document fits removes the only pan affordance and
contradicts the required always-visible behavior.

**Migration**: Remove the `ScrollBarAsNeeded` visibility policy and the
visibility-settling two-pass loop; both bars are always visible and remain a
projection of `offset_`/`zoom_`. The projection requirement and the pan clamp are
unchanged.
