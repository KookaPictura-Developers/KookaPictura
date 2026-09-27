# Proposal: magnetic-lasso-click-deselect

## Why

A click outside the selection with a selection tool deselects in CS6 and
photorust (#96, PR #97 for the other selection tools). The Magnetic Lasso,
added on this branch, needs the same rule.

## What Changes

- A Magnetic Lasso outline closed with fewer than three points (a double-click
  on one spot) runs Deselect in New mode when a selection exists.
- The `magnetic_lasso` self-test (530) covers it.

## Capabilities

### Modified Capabilities

- `tools/magnetic-lasso`: adds the click-to-deselect requirement.

## Impact

- `tool_magneticlasso.cpp`, `selftest_magnetic_lasso.{h,cpp}`. No new dependency.
