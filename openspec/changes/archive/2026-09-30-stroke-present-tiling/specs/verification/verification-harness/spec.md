# Spec Delta

## ADDED Requirements

### Requirement: Paint present-tiling self-check

The C++ self-test SHALL include a check, with the next free append-only exit
code, that paints several dabs of one stroke between two presents and asserts
that the view pyramid is rebuilt at most twice across the frame — strictly fewer
times than dabs — and that the committed pixels equal the same stroke committed
through the CPU path. The check SHALL cover the GPU path when an adapter is
present and the frame-bounded exact path otherwise, and SHALL report PASS with
the observed dab and rebuild counts.

#### Scenario: The frame rebuild is observed [vh_pyramid_selfcheck]

- **WHEN** the paint present-tiling self-check runs
- **THEN** it reports the dab count and a view-pyramid rebuild count no larger
  than twice the frame count and smaller than the dab count

#### Scenario: The committed pixels match the CPU path [vh_pyramid_commit]

- **WHEN** the same stroke is committed on the checked path and on the CPU path
- **THEN** their committed images are byte-identical

#### Scenario: The collapsed commit blit equals the level-0 crop [vh_commit_blit]

- **WHEN** the unit check commits a single-region stroke and compares its blit
  image with the level-0 crop of the same rectangle
- **THEN** every pixel channel is identical
