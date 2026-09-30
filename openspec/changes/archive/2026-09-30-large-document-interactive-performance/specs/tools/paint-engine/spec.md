# Spec Delta

## MODIFIED Requirements

### Requirement: Live paint stroke stays interactive on large documents

The live paint stroke SHALL stay interactive on large documents. While a paint
stroke is in progress the app SHALL update only the affected region and SHALL
keep the per-dab work bounded, free of whole-document costs: the command
registry, window/tab titles, and cursor policy SHALL NOT be recomputed on every
dab (they run once when the stroke commits), and the full-resolution canvas SHALL
NOT be detached or fully rebuilt per dab. The in-stroke region composite SHALL
use whichever backend is faster for the document (measured on the reference
machine: the GPU region composite is faster than the CPU oracle for a 512 px
brush dab). Releasing the mouse SHALL still commit exactly one history state and
SHALL refresh only the stroke's changed rectangle (its composite matches a full
recomposite) while the panels, registry, and titles refresh once. Cancelling a
stroke SHALL likewise restore the changed rectangle from the unchanged document
without a full-document recomposite.

#### Scenario: Per-dab GUI work is coalesced [pe_dab_no_registry_refresh]

- **WHEN** many dabs are applied during one stroke
- **THEN** the command registry and window/tab titles are not refreshed per dab,
  and exactly one history state is recorded on release

#### Scenario: The commit runs the full refresh [pe_commit_refresh]

- **WHEN** the stroke is released
- **THEN** the stroke's changed rectangle is refreshed so the composite matches a
  full recomposite, no full-document composite runs, the panels/registry/titles
  refresh once, and exactly one history state exists
