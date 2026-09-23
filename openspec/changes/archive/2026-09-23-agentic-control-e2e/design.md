## Context

`--self-test`'s `mcp_control` block calls `ControlServer::dispatch` directly, so
it never exercises `QLocalServer`/`QLocalSocket`, the newline framing, or the
separate process. `scripts/verify-control.sh` closes that: a throwaway
`pictura --headless --control` process on a temp socket, driven over the real
protocol. No new dependency: the client is `python3`'s stdlib `socket`, the same
runtime the oracle scripts use.

## Goals / Non-Goals

- **Goal:** an automated live end-to-end proof of the control endpoint, runnable
  by `verify-full.sh` and CI.
- **Non-goal:** a generic MCP client or a second control-server implementation.
  The script is one recipe, not a framework.
- **Non-goal:** pixel-exact filter values. The app's filter seed is fixed, so the
  loop asserts *changed* vs *unchanged* pixels, not absolute colours, keeping
  the check stable across filter tweaks.

## Decisions

### Process isolation and teardown

A `mktemp -d` supplies `--control-socket "$tmp/control.sock"` and
`--state-home "$tmp/state"`. The app is launched in the background with
`--headless` (offscreen: no display) and the fixture as the positional document,
redirecting stderr to a log. A `trap ... EXIT` kills the process, waits, and
removes the temp dir, so a failure never leaks a process or socket.

### Readiness

The script polls for the socket path to exist (bounded, e.g. 10 s) before
connecting, and fails loudly with the stderr log if it does not. A Python client
sets a socket timeout so a wedged request fails the gate instead of hanging it.

### The recipe

One connection, newline-delimited JSON, each response matched by `id`:

1. `status` → exactly one document, 8x8.
2. `selection` `op=rect`, `x=4`, `y=4`, `w=2`, `h=4` → 8 px. The active layer's
   rect is `4,4..8,8`, so the selection is a strict subset of the layer.
3. read all 64 pixels, `filter` `kind=add-noise`, read them again: at least one
   pixel changed and **every** changed pixel lies inside the selection. Because
   the selection is a subset of the active layer, a filter that ignored the mask
   would change pixels outside it and fail (the in-process self-test does not
   cover this).
4. `edit` `op=undo` → all 64 pixels equal the before snapshot.
5. `screenshot` `scope=document` → `mime` is `image/png` and the base64 decodes
   to a PNG signature; `source_width`/`source_height` are 8.

Any failed assertion exits non-zero with the offending step.

### Wiring

`scripts/verify-full.sh` already builds the binary, so it runs
`bash scripts/verify-control.sh` after `verify-fast.sh`. `verify-fast.sh` stays
build-free and does not run it. A missing `./build/pictura` is a hard error in
the script (not a skip): the gate that calls it builds first.

## Risks / Trade-offs

- The fixture path is repo-relative; the script `cd`s to the repo root like the
  other scripts and fails if the fixture is absent.
- `--headless` sets the offscreen QPA before `QApplication`; `grab()` works
  offscreen, so the screenshot step is valid without a display. CI already runs
  the app headless.
