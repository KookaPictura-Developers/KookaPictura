# Tasks: paste-keeps-source-position

## 1. Bridge

- [x] 1.1 `clipboard_paste`: place Plain at `clip.rect` (with InPlace), move the
  active selection into `deselected_selection` after every successful paste, and
  refresh the doc comment.
- [x] 1.2 Update the `frame_menus_edit.cpp` paste comment.

## 2. Verification

- [x] 2.1 Extend `tst_edit_clipboard.cpp`: a plain paste lands at the copied
  rect and drops the marquee.
- [x] 2.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cmake --build build --parallel`;
  `ctest --test-dir build -R '^tst_edit_clipboard$' --output-on-failure`;
  `bash scripts/verify-fast.sh`; `openspec validate --all --strict`.
