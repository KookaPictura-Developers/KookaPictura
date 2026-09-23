# agentic-control Specification

## Purpose
TBD - created by archiving change app-control-server. Update Purpose after archive.
## Requirements
### Requirement: Opt-in control mode over a local socket

The application SHALL start a control server only when invoked with `--control`.
The server SHALL listen on a per-user Unix-domain socket whose default path is
`$XDG_RUNTIME_DIR/pictura-control.sock`, overridable with `--control-socket
PATH`; the resolved path SHALL be printed to stderr. `--state-home DIR` SHALL
isolate the session store to `DIR`. `--headless --control` SHALL run offscreen
without a display. Without `--control`, no socket SHALL be created and startup,
the self-test, and normal runs SHALL be unchanged.

#### Scenario: Control mode starts and reports its socket

- **WHEN** the app is launched with `--control`
- **THEN** it creates the socket, prints `pictura control: <path>` to stderr, and
  a client that connects receives a `status` response

#### Scenario: No control server without the flag

- **WHEN** the app is launched without `--control`
- **THEN** no socket path is printed and the default socket is not created

#### Scenario: A custom socket path is honored

- **WHEN** the app is launched with `--control --control-socket /tmp/x.sock`
- **THEN** the server listens on `/tmp/x.sock`

### Requirement: Newline-delimited JSON protocol

The server SHALL read one JSON object per line. A request SHALL be
`{"id":<int>,"method":<string>,"params":<object>}`. A success response SHALL be
`{"id":<int>,"ok":true,"result":<object>}`; a failure response SHALL be
`{"id":<int>,"ok":false,"error":{"code":<string>,"message":<string>}}` and SHALL
echo the request's integer `id` whenever the request carried one (an unparseable
line has no id to echo). JSON SHALL be serialized compact so a message never
contains a raw newline. A malformed or unparseable line SHALL produce a
`bad_request` error and SHALL NOT crash the application. The error codes SHALL be
`bad_request`, `unknown_method`, `invalid_param`, `no_document`,
`not_implemented`, `refused`, `io_error`, and `internal`.

#### Scenario: A malformed line does not crash the app

- **WHEN** a client sends a line that is not valid JSON
- **THEN** the server responds with an error whose code is `bad_request` and the
  process stays alive

#### Scenario: An unknown method is reported

- **WHEN** a client sends `{"id":7,"method":"no_such_method","params":{}}`
- **THEN** the response has `id` 7, `ok` false, and error code `unknown_method`

#### Scenario: A document method with no document is reported

- **WHEN** a document-requiring method is invoked while no document is open
- **THEN** the response has error code `no_document`

### Requirement: Non-interactive control mode

Control mode SHALL set the unsaved-prompt policy to non-interactive with the
Discard choice at startup. The `document` method SHALL use the non-dialog entry
points and SHALL NOT reach `showOpenDialog`, `showNewDocumentDialog`, or any
other modal; in particular a `save` with no path on a document without a file
path SHALL return `invalid_param` rather than opening Save-As. Because
`dispatch_command` invokes the real menu handlers, some of which open modal
dialogs, it SHALL refuse a command that opens a modal dialog (or that can reach
the unsaved prompt) with the `refused` error and SHALL NOT invoke its handler.
Consequently, no control request SHALL block on a modal dialog.

#### Scenario: Opening a document never opens a file dialog

- **WHEN** a client invokes `document` with op `open` and a path
- **THEN** the document opens without a modal dialog and the response reports the
  new active index and count

#### Scenario: A dirty-document close does not prompt

- **WHEN** a document is dirty and a client invokes `document` with op `close`
- **THEN** the document closes without a modal, per the non-interactive Discard
  policy

#### Scenario: Saving an untitled document is refused

- **WHEN** a client invokes `document` with op `save` on a document with no file
  path
- **THEN** the response is `invalid_param` and no Save dialog opens

#### Scenario: A modal-opening command is refused

- **WHEN** a client invokes `dispatch_command` with a command that opens a modal
  dialog
- **THEN** the response is `refused` and the handler does not run

### Requirement: State and inspection readback

The server SHALL provide these read-only methods returning a JSON object:
`status` (document count, active index, per-document name/path/width/height/
mode/depth/dirty/layer-count/selection-count, active tool, backend,
gpu-available, brightness, screen mode, panel names/visibility, zoom);
`get_pixel` (`x`, `y` → the active document's composited `0xAARRGGBB` plus the
channel values); `list_layers` (per layer index/name/kind/visible/opacity/blend/
fill/lock/color); and `list_commands` (per command id/path/label/implemented/
enabled/checked, optionally restricted to implemented commands).

#### Scenario: Status describes the active document

- **WHEN** a client invokes `status` with one open document
- **THEN** the result reports one document, its dimensions, mode, depth, and
  layer and selection counts

#### Scenario: A pixel can be read back

- **WHEN** a client invokes `get_pixel` at a coordinate inside the document
- **THEN** the result carries the composited ARGB and channel values at that
  coordinate

#### Scenario: Layers and commands are enumerable

- **WHEN** a client invokes `list_layers` and `list_commands`
- **THEN** the results enumerate the active document's layers and the command
  registry with each command's implemented/enabled/checked state

### Requirement: Non-interactive action surface

The server SHALL provide `dispatch_command` (`id` → whether it dispatched and
its resulting enabled state, then a refresh; a modal-opening command is refused
as above), `document` (op `open`/`new`/`save`/`save_as`/`revert`/`close`/
`activate`/`list` with the op-specific parameters; `open` accepts both a native
PSD/PSB path and a raster image path, matching the GUI), `edit` (op
`undo`/`redo` → `success` plus can-undo/can-redo/history depth), and
`set_unsaved_policy` (`interactive` and optional `choice` = `cancel`/`discard` →
the resulting policy). A mutation SHALL be applied through the same paths the
application uses for its own commands. When `set_unsaved_policy` has re-armed
the interactive prompt, `dispatch_command` SHALL refuse the commands that can
reach that prompt.

#### Scenario: Dispatching a command performs its action

- **WHEN** a client invokes `dispatch_command` with an implemented command id
- **THEN** the command's action runs and the response reports it dispatched

#### Scenario: Undo and redo round-trip

- **WHEN** a mutation is made and a client invokes `edit` with op `undo` then
  `redo`
- **THEN** each reports success and the history depth reflects the change

#### Scenario: The unsaved policy can be flipped

- **WHEN** a client invokes `set_unsaved_policy` with `interactive` true and
  choice `cancel`
- **THEN** the response reports the new interactive state and choice

### Requirement: Local-only security

The socket SHALL be a Unix-domain socket created with user-only access (owner
read/write and no group or other access; Qt's `UserAccessOption` creates it with
mode `0700`), SHALL NOT be a TCP listener, and the control surface SHALL NOT
include any arbitrary-code-evaluation method. The control server SHALL exist
only under `--control`. A line or buffered request larger than a fixed bound
SHALL be rejected with `bad_request` rather than growing without limit.

#### Scenario: The socket is user-only

- **WHEN** the control server creates its socket
- **THEN** the socket's permissions grant access to the owning user only

#### Scenario: No eval method exists

- **WHEN** a client attempts a method that would evaluate arbitrary code
- **THEN** the response is `unknown_method`

#### Scenario: An oversized request is rejected

- **WHEN** a client sends an unterminated request that exceeds the fixed limit
- **THEN** the server responds with `bad_request` and does not grow the buffer
  without bound

### Requirement: Verification of the control server

The self-test SHALL include a `mcp_control` block that starts a control server on
a temporary socket, connects a client, and round-trips at least `status`,
`list_commands`, and a `dispatch_command`, asserting the response shapes and the
`bad_request`/`unknown_method` error paths, using the next append-only check
code.

#### Scenario: The control block proves the socket and core methods

- **WHEN** the self-test runs the `mcp_control` block
- **THEN** a client connects to the temporary socket and receives well-formed
  `status`, `list_commands`, and `dispatch_command` responses, and a malformed
  line yields `bad_request`

### Requirement: Screenshot readback

The server SHALL provide a `screenshot` method with a `scope` of `window`,
`canvas`, or `document` (default `window`) and an optional `max_dim` (default
1280). It SHALL return `{mime:"image/png", base64, width, height, source_width,
source_height}`: the PNG of the whole frame (`window`), the active canvas widget
including its checkerboard/zoom/overlay (`canvas`), or the raw composited
document image (`document`). When the image's long edge exceeds `max_dim` it
SHALL be scaled down preserving aspect ratio, and the pre-scale dimensions SHALL
be reported as `source_width`/`source_height`; `width`/`height` are the returned
image's. `canvas` and `document` with no active document SHALL return
`no_document`. A non-positive `max_dim` SHALL be `invalid_param`.

#### Scenario: A window screenshot is a PNG

- **WHEN** a client invokes `screenshot` with `scope` `window`
- **THEN** the result's `mime` is `image/png`, its `base64` decodes to a PNG, and `width`/`height` match the decoded image

#### Scenario: A canvas screenshot with no document is reported

- **WHEN** a client invokes `screenshot` with `scope` `canvas` and no document is open
- **THEN** the response has error code `no_document`

#### Scenario: max_dim downscales and reports the source size

- **WHEN** a client invokes `screenshot` with a `max_dim` smaller than the source long edge
- **THEN** the returned image's long edge is at most `max_dim`, `width`/`height` are the returned image's dimensions, and `source_width`/`source_height` are the original ones

#### Scenario: A non-positive max_dim is rejected

- **WHEN** a client invokes `screenshot` with `max_dim` 0 or negative
- **THEN** the response has error code `invalid_param`

### Requirement: UI tree readback

The server SHALL provide a `ui_tree` method with optional `max_depth` (default
12) and `max_children` (default 64) limits. It SHALL return a depth- and
breadth-capped array of nodes, each `{class, objectName, rect:{x,y,w,h},
visible, enabled, text, tooltip}`, where `rect` is in window (frame) coordinates.
A default call SHALL include the layers panel's object name so an agent can
locate it (the panel sits ~9 widget levels below the frame, well within the
default depth).

#### Scenario: The default tree contains the layers panel

- **WHEN** a client invokes `ui_tree` with no parameters
- **THEN** some node's `objectName` is `layersPanel` and its `rect` is a window-local rectangle

#### Scenario: The walk is bounded

- **WHEN** a client invokes `ui_tree` with small `max_depth`/`max_children`
- **THEN** the returned tree does not exceed those bounds and the server responds without error

### Requirement: Layer thumbnail readback

The server SHALL provide a `layer_thumbnail` method with `index` and an optional
`size` (default 64). For a layer that has a thumbnail it SHALL return the same
image shape as `screenshot` (`mime`, `base64`, `width`, `height`,
`source_width`, `source_height`); for a group, an adjustment layer, or an
out-of-range index it SHALL return `invalid_param`.

#### Scenario: A pixel layer has a thumbnail

- **WHEN** a client invokes `layer_thumbnail` for a pixel layer
- **THEN** the result's `mime` is `image/png` and its `base64` decodes to a PNG of at most `size` on the long edge

#### Scenario: A group layer has no thumbnail

- **WHEN** a client invokes `layer_thumbnail` for a group layer
- **THEN** the response has error code `invalid_param`

### Requirement: Vision verification

The self-test SHALL include a block that exercises the three vision methods over
the control socket, using the next append-only check codes. It SHALL assert that
a `screenshot` is a PNG and that its reported dimensions match the decoded image,
that a downscaled `screenshot` reports the source dimensions, that `ui_tree`
contains the layers panel, and that a pixel layer's `layer_thumbnail` is a PNG
while a group's is `invalid_param`.

#### Scenario: The vision block proves the methods offscreen

- **WHEN** the self-test runs the vision block under `--headless`
- **THEN** every vision assertion passes and the block reports pass, using no display

