## MODIFIED Requirements

### Requirement: Layer effects are rejected before GPU dispatch

A visible layer carrying a decodable layer effect SHALL make `composite_gpu`
return `GpuError::UnsupportedLayerEffect` before dispatching that layer to the
GPU, without panicking. A layer counts as effect-bearing when the effects the
compositor resolves for it decode to an enabled and present `DropShadow`, an
enabled and present `OuterGlow`, an enabled and present `InnerShadow`, an
enabled and present `InnerGlow`, an enabled and present `BevelEmboss`, an enabled
and present `Satin`, an enabled and present `Stroke` (solid, gradient, or
pattern fill), an enabled and present `ColorOverlay`, an enabled and present
`GradientOverlay`, or an enabled and present `PatternOverlay`. The effects SHALL
be resolved through the same shared decode path the CPU compositor uses: an
`lfx2` block, or, when `lfx2` is absent, the layer's legacy `lrFX`
(`EFFECTS_LAYER`) block mapped into the typed effect set, with `lfx2` taking
precedence when both are present. A disabled, absent, or malformed effect, a
bevel whose style or technique is not rendered, a stroke whose gradient or
pattern fill payload cannot be decoded, a legacy effect that maps to no renderer,
an overlay whose pattern or gradient payload cannot be decoded, and a legacy
block that does not decode SHALL NOT reject the document. `composite_active` and
`composite_gpu_or_cpu` SHALL fall back to the CPU composite for a document with
such a layer, and the fallback output SHALL be byte-identical to `composite_rgba`
of the same document.

#### Scenario: A drop-shadow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present drop shadow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: An outer-glow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present outer glow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: An inner-shadow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present inner shadow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: An inner-glow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present inner glow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: A bevel and emboss layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present bevel and emboss
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: A satin layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present satin
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: A solid stroke layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present solid-colour stroke
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: A gradient or pattern stroke layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present gradient-fill stroke, or an enabled and present pattern-fill stroke
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: An overlay layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present color overlay, gradient overlay, or pattern overlay
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: A legacy effects layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries only a legacy `lrFX` block decoding to an enabled and present drop shadow, outer glow, inner shadow, inner glow, bevel, or solid fill
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: The effect document falls back to the CPU composite

- **WHEN** `composite_gpu_or_cpu` is called on a document whose visible layer carries an enabled and present object-based or legacy effect
- **THEN** it returns the same buffer as `composite_rgba` for that document

#### Scenario: A disabled or undecodable effect does not reject the GPU

- **WHEN** `composite_gpu` is called on a document whose only effect is disabled, whose stroke gradient/pattern payload cannot be decoded, whose overlay pattern/gradient payload cannot be decoded, whose legacy block is malformed or maps to a bevel style that is not rendered, or whose legacy block resolves to no effect
- **THEN** the effect does not by itself produce `UnsupportedLayerEffect`
