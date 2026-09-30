# Spec Delta

## ADDED Requirements

### Requirement: Tile-aligned damage update

The view structure SHALL round a damage rectangle out to the 64×64 tile grid
before recomputing the levels below 0, then expand it outward per level by the
2× filter footprint a level-`N` texel reads from `N-1`. The aligned update MUST
remain byte-identical to a full rebuild, and level 0 MUST stay the caller's
single supplied buffer — the alignment MUST NOT copy or detach a full-frame
buffer.

#### Scenario: A non-aligned damage rect still equals a rebuild [vp_tile_update]

- **WHEN** an update is called with a rectangle whose edges are not tile-aligned
- **THEN** every level below 0 equals a rebuild of the same level-0 pixels

#### Scenario: The damage rect is rounded outward [vp_tile_align]

- **WHEN** a damage rectangle of 10×20 at (130, 70) is aligned on a 64-pixel grid
- **THEN** it becomes the rectangle (128, 64)–(192, 128), covering the original
  rectangle

#### Scenario: Level 0 is not detached [vp_tile_no_detach]

- **WHEN** a tile-aligned update runs
- **THEN** level 0 is read from the supplied buffer and no full-frame copy is
  made
