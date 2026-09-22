## Context

The design is `docs/dev/mcp-agentic-control-plan.md` (authored before coding so
the work can resume cold); this change implements its P0/P1 slice. The app is a
Qt C++ shell (`crates/pictura-app/cpp/`) around a Rust `staticlib`
(`crates/pictura-app/src/cxxqt_object/*`, exposed as the `PictureView` QObject).
`main.cpp` already has an in-process, non-interactive test harness
(`runSelfTest`, `cpp/selftest*.cpp`) with a stable append-only check-code
convention, and the non-dialog document entry points the harness uses.

## Goals / Non-Goals

**Goals:**
- Start an opt-in, local-only JSON control endpoint from `main.cpp` with no
  effect on normal runs or `--self-test`.
- Expose the read-only state an agent needs to orient: `status`, `get_pixel`,
  `list_layers`, `list_commands`.
- Expose a minimal non-interactive action surface: `dispatch_command`,
  `document`, `edit`, `set_unsaved_policy`.
- Never block on a modal in control mode.
- Prove the socket, framing, and the core methods with one runnable self-test.

**Non-Goals:**
- No MCP frontend (`pictura-mcp`), no `rmcp`/`tokio`; those are a later change.
- No vision (`screenshot`/`ui_tree`/`layer_thumbnail`), no input synthesis
  (`pointer`/`key`/`set_tool`), no engine actions (`selection`/`filter`/
  `adjustment`/`layer_op`/`set_gpu_compute`).
- No change to the engine/codec, and no new Rust dependency.
- No TCP listener, no eval tool, no remote transport.

## Decisions

- **Two processes eventually, one now.** The plan splits an in-app socket
  endpoint from an out-of-process MCP frontend so `tokio`/`rmcp` never enter the
  app staticlib and dispatch stays on the GUI thread. This change is only the
  in-app endpoint; it is useful on its own to any local client and unblocks the
  frontend.
- **`QLocalServer` on the GUI thread.** `newConnection`/`readyRead` run on the Qt
  event loop, so a request handler calls `PicturaMainWindow`/`PictureView`
  directly — no locks, no cross-thread marshalling. Long operations block the
  call and return, matching the app's synchronous model today (ceiling:
  `ponytail:` long composites block the socket; upgrade path is app-wide
  off-GUI-thread compute, a separate change).
- **A small dispatcher class, `ControlServer`** (`control_server.{h,cpp}`), owns
  the server, per-socket read buffers, and one `handle(method, params) ->
  result-or-error` function. Keeping the JSON dispatch in one place makes the
  methods testable and the framing trivial. It is constructed in `main()` after
  the frame and before `app.exec()` only when `--control` is set.
- **A `CommandRegistry` enumeration accessor.** `list_commands` needs the command
  table, which is private today (`commands.h:177-181`); a new
  `QList<CommandInfo> describe() const` (or equivalent) exposes
  `id`/`path`/`label`/`implemented` plus `enabled`/`checked` resolved from the
  `QAction` (there is no `isEnabled` on the registry). Minimal new surface, used
  by both `list_commands` and `dispatch_command`.
- **Non-interactive by default.** Control startup calls the same
  `setUnsavedPromptInteractive(false)` + Discard the self-test uses, and the
  `document` method calls the non-dialog entry points (`frame.openPath`,
  `frame.newDocument`, …) so no modal path is reachable.
- **Self-test at the next free code (463), not the plan's 128/129.** Check codes
  are append-only and 129 was retired; the current maximum is 462. The block
  starts its own `ControlServer` on a temporary socket path and round-trips
  requests through a `QLocalSocket`, driving the event loop with a short
  `QEventLoop`/`QTimer`, so it does not depend on the main-run instance.
- **Socket permissions and path.** `QLocalServer::UserAccessOption` (owner-only;
  Qt creates the socket with mode `0700`); default
  `$XDG_RUNTIME_DIR/pictura-control.sock`; `--control-socket` overrides;
  `--state-home` isolates the XDG session store (like the self-test temp dir).
  `--state-home` isolates the XDG session store (like the self-test temp dir).

## Risks / Trade-offs

- **First IPC in the codebase.** No existing `QLocalServer`/`QLocalSocket` to
  imitate; the framing is deliberately tiny (one compact JSON object per line)
  to keep it auditable.
- **Blocking the GUI thread during long calls.** Accepted for a test surface;
  documented as a ceiling.
- **File access equals the app's.** `document open|save` can touch anything the
  user can; documented, with a future `--control-root` allowlist as the
  hardening path if agents ever run untrusted.
- **Command availability drifts.** `list_commands` reports `implemented` /
  `enabled` from the live registry so an agent sees the real state rather than a
  hardcoded list.

## Migration

None. The endpoint exists only with `--control`; every existing flag, the
self-test, and normal startup are unchanged.
