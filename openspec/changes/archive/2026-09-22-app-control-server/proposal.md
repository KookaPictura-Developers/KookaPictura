## Why

Today the app can only be verified through `--self-test` exit codes: one long
in-process oracle that cannot inspect UI state, read a pixel back, or drive a
real command against a running instance. The project's own plan
(`docs/dev/mcp-agentic-control-plan.md`) calls for an opt-in control endpoint so
an agent (or a script) can see and drive a live Pictura. This change ships the
first slice: the app-side control server. A future change adds the `pictura-mcp`
stdio frontend that speaks MCP over this socket.

## What Changes

- `--control [--control-socket PATH] [--state-home DIR]` starts a `QLocalServer`
  on a per-user socket (`0600`, default `$XDG_RUNTIME_DIR/pictura-control.sock`),
  prints the resolved path to stderr, and enters the normal event loop. Without
  `--control` nothing changes; `--headless --control` runs offscreen.
- A newline-delimited JSON protocol: request
  `{"id":<int>,"method":"<string>","params":{...}}`; success
  `{"id":<int>,"ok":true,"result":{...}}`; failure
  `{"id":<int>,"ok":false,"error":{"code":"<enum>","message":"<text>"}}`. JSON is
  serialized compact so a message never contains a raw newline. A malformed line
  yields `bad_request` and never crashes the app.
- Control methods (app side, dispatched on the GUI thread):
  `status`, `get_pixel`, `list_layers`, `list_commands`, `dispatch_command`,
  `document`, `edit`, `set_unsaved_policy`.
- Control mode is non-interactive: it sets the unsaved-prompt policy to
  non-interactive / Discard at startup, and its `document open|new|save|…` use
  the same non-dialog entry points as `--self-test`, so automation never blocks
  on a modal.
- The app links `Qt6::Network` (system Qt, already installed); no new Rust
  dependency and no engine/codec change.
- Deferred to later changes (explicitly out of scope here): vision
  (`screenshot`/`ui_tree`/`layer_thumbnail`), input synthesis
  (`pointer`/`key`/`set_tool`), engine actions
  (`selection`/`filter`/`adjustment`/`layer_op`/`set_gpu_compute`), and the
  `pictura-mcp` MCP frontend.

## Capabilities

### New Capabilities
- `agentic-control`: an opt-in, local-only, JSON-over-Unix-socket control
  endpoint that exposes read-only application/document/layer state and a
  non-interactive action surface (command dispatch, document, edit, unsaved
  policy) for manual and agentic testing of a running instance.

## Impact

- `crates/pictura-app/cpp/`: new `control_server.{h,cpp}`; `main.cpp` for the CLI
  flags and server lifecycle; `commands.{h,cpp}` for a command-list accessor;
  `CMakeLists.txt` to find `Qt6` `Network`, link `Qt6::Network`, and list the new
  sources (there is no globbing).
- Self-test: a new `mcp_control` block proving the socket, framing, and the core
  methods in-process.
- No engine or codec change; no new Rust dependency; the long-form docs
  (`docs/11-cross-cutting/agentic-testing.md`, `AGENTS.md`) land in a separate
  `TASK-ALLOWS-DOCS` commit.
