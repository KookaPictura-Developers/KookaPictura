## 1. Script

- [x] 1.1 Add `scripts/verify-control.sh` (executable): launch
      `./build/pictura --headless --control --control-socket <tmp> --state-home
      <tmp> <fixture>` in the background, poll for the socket, drive the recipe
      from a `python3` stdlib-socket client, tear down on exit.
- [x] 1.2 Assert the recipe (status 1×8x8; a rect selection that is a strict
      subset of the active layer; the filter changed ≥1 pixel and none outside
      the selection; undo restores all 64 pixels; screenshot is a document PNG
      with 8x8 source) and exit non-zero on any mismatch or a missing binary.

## 2. Wiring

- [x] 2.1 `scripts/verify-full.sh`: run `bash scripts/verify-control.sh` after
      `verify-fast.sh` (the build is already done).
- [x] 2.2 `.github/workflows/ci.yml` `qt-headless`: run
      `bash scripts/verify-control.sh` after the build.

## 3. Verification

- [x] 3.1 Run `bash scripts/verify-control.sh` on a built tree; it exits 0 and
      leaves no process/socket behind.
- [x] 3.2 Revert-proof: stub out the selection call (a mask-ignoring filter) and
      confirm the script fails with the outside pixels, then restore.
- [x] 3.3 `bash scripts/verify-full.sh` green; `openspec validate
      agentic-control-e2e --strict`.
