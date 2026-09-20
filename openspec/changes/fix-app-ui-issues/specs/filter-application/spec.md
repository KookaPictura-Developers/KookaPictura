## MODIFIED Requirements

### Requirement: Destructive filter application entry point

`pictura-render` SHALL expose `pub fn apply_filter(layer: &mut pictura_core::Layer, filter: &pictura_filters::Filter, mask: Option<&pictura_core::LayerMask>) -> Result<(), pictura_filters::FilterError>`. It SHALL apply `filter` destructively to the layer's color channels and gate the write by the optional document-coordinate `mask`. When `layer.rect.width()` or `layer.rect.height()` is `<= 0` it MUST return `Ok(())` without modifying the layer. When the layer's lock state includes the pixel lock, it MUST return a refusal error without modifying the layer.

#### Scenario: A non-empty layer is filtered in place

- **WHEN** `apply_filter` is called on a layer with a non-empty `rect` and a valid filter
- **THEN** the layer's color channels are rewritten in place and `Ok(())` is returned

#### Scenario: A degenerate rect is a no-op

- **WHEN** `apply_filter` is called on a layer whose `rect` has zero or negative width or height
- **THEN** it returns `Ok(())` and the layer is bit-identical to its input

#### Scenario: A pixel-locked layer is refused [lfa_pixel_locked]

- **WHEN** `apply_filter` is called on a layer whose lock state includes the pixel lock
- **THEN** it returns a refusal error and the layer's channels are bit-identical to their input
