## Why

The control server is proven only by the in-process `--self-test` block, which
runs the dispatch logic directly. Nothing drives the real out-of-process
endpoint — a separate `pictura --headless --control` process, a Unix-socket
connection, and a request/response loop — so the framing, the socket lifecycle,
and the full open→select→filter→undo→screenshot recipe are unverified end to end.
The plan (`docs/dev/mcp-agentic-control-plan.md` §13 P5) specifies a
`scripts/verify-control.sh` for exactly this.

## What Changes

- Add `scripts/verify-control.sh`: launch `pictura --headless --control` on a
  temp socket, connect, and drive the documented recipe — open the fixture, apply
  a rectangular selection that is a strict subset of the active layer, apply a
  seeded filter, assert it changed at least one pixel and no pixel outside the
  selection, undo and assert every pixel is restored, screenshot — then shut the
  app down and remove the socket. Exit non-zero on any mismatch or a missing
  build.
- Wire it into `scripts/verify-full.sh` after the build (it needs the binary) and
  into the CI headless job, so the live loop is part of both gates. `verify-fast.sh`
  stays build-free.
- No new dependency (the client is `python3`'s stdlib `socket`, already used by
  the oracle scripts); no new app capability.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `verification-harness`: add a requirement that a live control-server
  end-to-end script exists and that the full gate runs it.

## Impact

- New `scripts/verify-control.sh`; `scripts/verify-full.sh` gains one step.
- No Rust, C++, or CMake change.
