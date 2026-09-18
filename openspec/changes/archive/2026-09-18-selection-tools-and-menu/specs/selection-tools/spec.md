## MODIFIED Requirements

### Requirement: Magic Wand

`magic_wand(image, x, y, tolerance, contiguous)` SHALL build a selection from
the colour at the seed point. A pixel MUST be selected when its per-channel
maximum absolute difference (Chebyshev distance) from the reference colour is
less than or equal to `tolerance`. When `contiguous` is true, only pixels
four-connected to the seed MUST be selected; when false, every matching pixel in
the image MUST be selected. A seed outside the image, or an empty image, MUST
return an error. The enabled Magic Wand tool SHALL route a canvas click through
this function using the options-bar Tolerance (0-255) and Contiguous state, and
SHALL combine the result with the current selection using the selected New, Add,
Subtract, or Intersect mode. The tool SHALL offer Anti-alias and Sample All
Layers controls; because the engine rasteriser is binary and `current_buffer` is
the visible composite, both SHALL be visible and disabled with a reason.
Contiguous SHALL default to on and Tolerance to 32.

#### Scenario: Contiguous flood stops at a colour boundary

- **WHEN** the wand is seeded in a red region touching a blue region with a tolerance that excludes blue
- **THEN** only the connected red pixels are selected

#### Scenario: Global mode selects disconnected matches

- **WHEN** the wand runs non-contiguous on an image with two disconnected identical-coloured patches
- **THEN** pixels in both patches are selected

#### Scenario: Tolerance rejects distant colours

- **WHEN** a neighbouring pixel differs from the reference colour by more than `tolerance`
- **THEN** that pixel is not selected

#### Scenario: Out-of-bounds seed errors

- **WHEN** the seed coordinates fall outside the image
- **THEN** the wand returns an error instead of panicking

#### Scenario: The tool combines with the current selection

- **WHEN** a selection exists, Add mode is active, and the Magic Wand clicks a region
- **THEN** the committed selection is the union of the existing selection and the wand result in one undo state

#### Scenario: Unmodelled wand options are honest

- **WHEN** the Magic Wand options bar is shown
- **THEN** Anti-alias and Sample All Layers are disabled and state why they have no effect

## ADDED Requirements

### Requirement: Shared Magic Wand tolerance for Grow and Similar

The Magic Wand options-bar Tolerance SHALL be the single tolerance source for
`Select > Grow` and `Select > Similar`, so a Grow or Similar run after a wand
selection uses the value currently shown in the wand options bar. Grow and
Similar SHALL operate against the visible composite buffer and SHALL return an
error when the selection and buffer dimensions differ.

#### Scenario: Grow uses the shown tolerance

- **WHEN** the wand options bar shows Tolerance 40 and Select > Grow is run
- **THEN** growth accepts neighbours within 40 levels of the touched selected colour

#### Scenario: Dimension mismatch is refused

- **WHEN** the selection dimensions differ from the buffer and Select > Similar is run
- **THEN** the command is refused, the selection is unchanged, and no history state is recorded
