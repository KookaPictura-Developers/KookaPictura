## 1. Reserve the workspace

- [x] 1.1 Give `QTabWidget#documentTabs` a minimum width
  (`kWorkspaceMinWidth`, 160) and keep it in the central splitter at all times.
- [x] 1.2 In `frame.cpp::refresh`, keep the pane visible and hide only the empty
  tab strip when `docs_` is empty.
- [x] 1.3 Update `lss_empty_pane` (349) and `lss_fresh_pane` (380) to assert the
  reserved space and the hidden strip instead of a hidden pane.

## 2. Regression coverage

- [x] 2.1 Add `lpr_workspace_space` (400): with no document the workspace keeps
  its minimum width, the columns do not absorb the splitter, both workspace
  handles exist, an icon column is fixed at its strip width, and growing a normal
  column moves the workspace boundary by the same amount.

## 3. Verification

- [x] 3.1 `cmake --build build --parallel` and
  `./build/pictura --headless --self-test` — 335 passed, 0 failed.
- [ ] 3.2 `bash scripts/verify-full.sh`.
- [ ] 3.3 `openspec validate --all --strict`.

## 4. Not done

- [ ] 4.1 No dedicated empty-workspace placeholder widget; the document pane
  itself supplies the space.
- [ ] 4.2 No change to the interactive splitter resize ceiling in normal mode
  beyond the 400 px persistence bound.
