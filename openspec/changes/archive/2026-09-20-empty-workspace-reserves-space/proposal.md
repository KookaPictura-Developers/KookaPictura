## Why

With no document open the shell hid the whole `QTabWidget#documentTabs` pane.
A hidden splitter pane takes no space, so the widget columns absorbed the entire
central area: the workspace collapsed to zero width, the first right-hand column
had no grabbable left handle (its left neighbour was hidden), and splitter slack
went to the columns instead of the workspace. An icon-mode column could end up
with empty space beside its strip when the only visible neighbour could not
absorb the slack.

## What Changes

- Keep the document pane in the `centerSplitter` whenever the frame is
  constructed, with a minimum width and the stretch, so the columns can never
  absorb the workspace and each column keeps a grabbable inner handle.
- Hide only the empty tab strip when no document is open, not the whole pane.
- Update the empty-workspace checks to assert the reserved space and the hidden
  strip instead of a hidden pane, and add a regression check for the reserved
  workspace, the splitter handles, and the fixed icon strip.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `application-shell`: the empty document area keeps its workspace space and its
  splitter handles, without an empty tab strip.

## Impact

- **C++ app**: `crates/pictura-app/cpp/frame.cpp`,
  `crates/pictura-app/cpp/selftest_shell_round3.cpp`,
  `crates/pictura-app/cpp/selftest_session.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
