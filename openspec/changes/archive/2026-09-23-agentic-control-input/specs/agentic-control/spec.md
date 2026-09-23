## ADDED Requirements

### Requirement: Pointer input synthesis

The server SHALL provide a `pointer` method with an `op` of `click`, `dblclick`,
`move`, `drag`, or `scroll`, coordinates `x`/`y` (and `x2`/`y2` for `drag`), a
`space` of `window` or `image`, and optional `button`, `modifiers`, and `steps`.
It SHALL synthesize the corresponding `QMouseEvent`/`QWheelEvent` and deliver
them with `QApplication::sendEvent` through the application's real event path; it
SHALL NOT link or call a test framework. With `space` `image` the coordinates are
document pixels on the active `ImageView`, mapped by `widget = image * zoom +
offset`; with no active document it SHALL return `no_document`. With `space`
`window` the coordinates are frame-local (the space `ui_tree` reports). An
unknown `op` or `space`, a non-numeric coordinate, a `drag` without `x2`/`y2`, or
a `steps` outside its bound SHALL be `invalid_param`. It SHALL return `{ok}`.

#### Scenario: A drag in image space drives a tool

- **WHEN** a client selects a marquee tool and invokes `pointer` with op `drag`, space `image`, and a rectangle inside the document
- **THEN** the result is `ok` and a selection exists whose bounds cover the dragged rectangle

#### Scenario: Image space needs a document

- **WHEN** a client invokes `pointer` with space `image` and no document open
- **THEN** the response has error code `no_document`

#### Scenario: A malformed pointer request is rejected

- **WHEN** a client invokes `pointer` with an unknown `op`, or op `drag` without `x2`/`y2`
- **THEN** the response has error code `invalid_param`

### Requirement: Key input synthesis

The server SHALL provide a `key` method taking a `sequence` such as `Ctrl+Z`,
`B`, or `Shift+F2`. It SHALL parse the sequence into a modifier set and a key,
synthesize a `QKeyEvent` press and release, and deliver them with
`QApplication::sendEvent` to `QApplication::focusWidget()` when one exists, else
the frame. A synthesized (non-spontaneous) key press SHALL be routed through the
application's shortcut machinery, so an installed `QAction`/`QShortcut` whose
shortcut matches is triggered first, and a key that no installed shortcut
consumes reaches the widget's `keyPressEvent`. (A window-scoped shortcut matches
only while its window is active; the in-process self-test runs before the window
activates, so it exercises the widget-handler path, and a live run exercises the
shortcut path.) An unknown key name or modifier SHALL be `invalid_param`. It
SHALL return `{ok}`.

#### Scenario: An installed shortcut fires

- **WHEN** a client invokes `key` with a sequence bound to an installed shortcut in an active window (for example the screen-mode/brightness shortcut)
- **THEN** the result is `ok` and the shortcut's effect is observable in a later readback

#### Scenario: A key with no shortcut reaches the widget handler

- **WHEN** a client invokes `key` with a key that no installed shortcut consumes (for example an arrow key while the Move tool is active)
- **THEN** the result is `ok` and the widget's key-handler effect is observable in a later readback

#### Scenario: An unknown key is rejected

- **WHEN** a client invokes `key` with a sequence naming an unknown key
- **THEN** the response has error code `invalid_param`

### Requirement: Input verification

The in-process self-test SHALL drive a `pointer` drag and a `key` sequence
through the control server and assert the observable effect, so input synthesis
is checked offscreen without a display.

#### Scenario: The self-test drags a marquee

- **WHEN** the control self-test opens the fixture, selects the marquee tool, and sends a `pointer` drag in image space
- **THEN** it reports a committed selection with a non-zero pixel count and passes
