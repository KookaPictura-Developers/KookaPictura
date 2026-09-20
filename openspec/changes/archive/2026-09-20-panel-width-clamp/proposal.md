## Why

The round-3 persistence change let a normal-mode panel column persist a width it
had absorbed from the splitter (the workspace session held `railWidth=2521`,
columns 2477/2521). On the next launch that stored width restored as a
full-workspace panel, and the test that exercised the flip deliberately used an
oversized width, enshrining the bad value.

## What Changes

- Clamp a panel column's persisted and restored normal width to a sane range
  (180–400), so a stale or oversized store can never expand a column across the
  workspace.
- Keep the icon strip at its own narrow width instead of the normal-mode floor.
- Update the flip regression to assert the stored width is bounded, not the
  oversized value.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace-persistence`: the persisted/restored normal width is bounded.

## Impact

- **C++ app**: `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`,
  `selftest_session.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
