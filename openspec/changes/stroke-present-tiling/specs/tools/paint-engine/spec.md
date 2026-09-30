# Spec Delta

## ADDED Requirements

### Requirement: The live stroke presents and rebuilds the pyramid frame-bounded

While a stroke is live the app SHALL present at most once per frame: the dab
that opens a frame presents immediately and the frame's later dabs accumulate
into one pending region, shown at the frame flush. The view pyramid SHALL be
rebuilt once per presented frame rather than once per dab, and each stored level
MUST equal a rebuild of the frame's level-0 buffer.

#### Scenario: A frame of dabs rebuilds the pyramid once [pe_pyramid_per_frame]

- **WHEN** several dabs arrive between two presents of a live stroke
- **THEN** the view pyramid is rebuilt once, at the flush, rather than once per
  dab

#### Scenario: The refreshed pyramid matches a rebuild [pe_pyramid_flush_equal]

- **WHEN** a frame's accumulated region is folded into the pyramid
- **THEN** every stored level equals a rebuild from the frame's level-0 buffer
