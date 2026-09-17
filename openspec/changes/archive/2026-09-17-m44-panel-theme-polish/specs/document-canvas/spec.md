## ADDED Requirements

### Requirement: Initial new-document canvas render

A newly created document with a white background SHALL render pure white on its
first present, before any layer move, visibility change, or other recomposite.
The rendered canvas SHALL NOT show an uninitialized, stale, or otherwise
garbage frame on first use, and a subsequent recomposite SHALL produce the same
uniform result. The initial render SHALL use the same composite/readback seam as
every other composite, so the fix applies to the path all presenters route
through rather than to a single caller. The implementation SHALL first
reproduce the failure empirically, record the named root cause, and fix the
seam; the requirement SHALL be pinned by at least one runnable regression check
that composites a fresh white document twice and asserts both frames are uniform
white and equal.

#### Scenario: A new white document is pure white immediately

- **WHEN** a white-background document is created and its canvas is presented
  for the first time
- **THEN** every pixel of the presented canvas is fully opaque white, with no
  garbage frame

#### Scenario: A recomposite does not change the initial result

- **WHEN** the initial composite of a fresh white document is compared with a
  second composite of the same document
- **THEN** both frames are uniform white and equal

#### Scenario: The regression check fails on a garbage first frame

- **WHEN** the initial composite seam returns a non-uniform or non-white frame
  for a fresh white document
- **THEN** the regression check fails rather than passing vacuously
