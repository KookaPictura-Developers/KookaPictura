# Actions

- **Spec ID:** `AUTO-001`
- **Status:** `Draft`
- **Parity tier:** `Core` — the Actions panel, action sets, and playback exist in CS6 Standard and Extended.
- **New in CS6:** `Changed` — **Allow Tool Recording** lets brush strokes and other tools be recorded directly (CS5 could not); **conditional actions** (`Insert Conditional`) can branch on document state; Contact Sheet II and PDF Presentation return as Automate options. The Actions panel itself, action sets, modal controls, and `.atn` files are otherwise CS5-equivalent.
- **Depends on:** `ARCH-011` (plugin-and-scripting-abi), `AUTO-010` (rust-scripting-replacement), `01-architecture/undo-history`, `01-architecture/document-model`, `AUTO-002` (droplets), `AUTO-003` (batch-processing).

## CS6 behavior

An **action** is a recorded series of tasks (menu commands, panel options, tool actions) played back on one file or a batch of files. Actions are grouped into **action sets** and stored on disk as `.atn` files; they are the basis for **droplets** (`AUTO-002`) and the **Batch** command (`AUTO-003`). Photoshop ships predefined actions (`.atn` presets) that can be used, edited, or replaced.

### Recording

- Create with the **Create New Action** button or `New Action` from the panel menu: name, target set, optional **Function Key** shortcut, and a **Color** (Button-mode display colour). Recording begins at `Begin Recording`; the record button turns red. Stop with the Stop button, `Stop Recording`, or `Esc`.
- Under **recording**, commands/tools are appended in order. The observable recording surface spans menu commands and the Marquee, Move, Polygon, Lasso, Magic Wand, Crop, Slice, Magic Eraser, Gradient, Paint Bucket, Type, Shape, Notes, Eyedropper, and Color Sampler tools, plus History, Swatches, Color, Paths, Channels, Layers, Styles, and Actions panel operations.
- **Not recordable:** painting/toning tools, tool options, View commands, and Window commands. These are inserted afterwards with `Insert Menu Item` (the command then executes at playback, prompting its dialog if modal).
- Recorded dialog/panel settings are the values in effect at record time. Modal operations and position-recording tools use the current ruler units; recording at percentage units is the documented way to keep position relative across file sizes.
- `Save As` records the filename unless changed; leaving the filename unchanged and changing only the folder is the documented way to redirect output without baking in a name.
- Editing: double-click a command to overwrite its values; drag to rearrange within/across actions; `Record Again` re-runs the action with prompts to re-capture modal settings; `Start Recording` appends to the selected action.

### Special steps

| Step | Menu command | Behavior |
|---|---|---|
| Stop | `Insert Stop` | Pauses playback with a message and (optionally) an **Allow Continue** button. |
| Non-recordable command | `Insert Menu Item` | Inserts a command by choosing it from a menu while the dialog is open; no parameter values are recorded. |
| Path | `Insert Path` | Records the currently selected path; on playback the work path is set to it. A later `Insert Path` replaces an earlier one inside the same action (record a `Save Path` between them to keep several). |
| Conditional | `Insert Conditional` | `If Current <condition>` → `Then Play Action` / `Else Play Action`, each choosing an action from the same set or `None` (not both `None`). Common pattern: test the opposite by swapping Then/Else. |
| Conditional mode change | `File > Automate > Conditional Mode Change` | Records a source-mode set and a target mode as one action step, avoiding an error when the opened file is not the expected source mode. |

### Modal controls

A **modal control** pauses playback at a command/action/set so the user can enter values in its dialog or use a modal tool. Enabled/disabled by clicking the box left of the name; a red dialog icon on an action/set means *some but not all* its commands are modal. Modal controls cannot be set in Button mode, and a command inserted via `Insert Menu Item` that opens a dialog cannot have its modal control disabled.

### Playback

`Playback Options` sets one of three speeds:

- **Accelerated** (default) — normal speed; the screen may not update, so files open/modify/save without appearing.
- **Step By Step** — each command completes and the image redraws before the next.
- **Pause For __ Seconds** — a fixed pause between commands.

Playback can start at a set, an action, or a selected command; `Ctrl`/`Command`-click Play plays a single command. Commands excluded (unchecked) are skipped. In **Button mode**, clicking a button plays the whole action (excluded commands still skipped) but individual commands/sets are not shown and modal/exclude editing is unavailable.

### Managing sets

Actions can be rearranged, duplicated, deleted, renamed, and have their options changed. Sets can be created, saved (`Save Actions` → `.atn`), loaded, replaced, reset to defaults, or printed: `Ctrl+Alt`/`Command+Option` + `Save Actions` writes a human-readable text file that Photoshop cannot reload. A set placed in `Presets/Actions` appears at the bottom of the panel menu after restart.

### Action descriptor basis

Each recorded command is an **Action Manager** event: an *event ID* plus an *ActionDescriptor* of key/value parameters, executed with a dialog mode. The ScriptListener plug-in logs these as `charIDToTypeID` + `ActionDescriptor.putInteger/...` + `executeAction` source (`AUTO-004`). The event-data model (`ActionDescriptor`, `ActionList`, `ActionReference`) is shared with the scripting host and native plug-ins via the command layer (`ARCH-011`).

### `.atn` file

An action set file is a big-endian binary structure (community-documented and matching Adobe's *File Formats Specification* "Actions" section):

| Field | Size | Notes |
|---|---|---|
| Version | 4 | `16` |
| Action set | variable | Name (Unicode string), `expanded` (1), action count (4), then actions |
| Action | variable | Function-key number (2), Shift (1), Cmd/Ctrl (1), colour index (2, `0`–`7`), name (Unicode), `expanded` (1), command count (4), then commands |
| Command | variable | `expanded`, `enabled`, `withDialog` (1 each), dialog options (1), event-ID format (`'long'`/`'TEXT'`, 4), event ID, dictionary name (byte string), has-descriptor (4, `0`/`-1`), descriptor |

The embedded descriptor uses Adobe's OSType-tagged structure (`'obj '`, `'Objc'`, `'VlLs'`, `'doub'`, `'UntF'`, `'TEXT'`, `'enum'`, `'long'`, `'comp'`, `'bool'`, `'type'`/`'GlbC'`, `'alis'`, `'tdta'`), with a reference structure (`'prop'`, `'Clss'`, `'Enmr'`, `'rele'`, `'Idnt'`, `'indx'`, `'name'`). The Actions palette itself persists to `Actions Palette.psp` (`'8BPF'`, version 16) and is read only at launch, written at quit.

**Dialog options** are set at record time and combined with the user-toggleable `withDialog` flag to choose the `executeAction` dialog mode: `0`+false → `NO`, `0`+true → `ALL`, `1` → always `ALL`, `2` → always `NO`, `3` → always `ALL` (community-observed; `3`/`4` are undocumented).

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Window > Actions` | panel | `F9` | The Actions panel. |
| Actions panel buttons | toolbar | — | Create new action, Create new set, Play, Begin recording, Delete. |
| Actions panel menu | menu | — | `New Action…`, `New Set…`, `Duplicate`, `Delete`, `Play`, `Start/Stop Recording`, `Record Again…`, `Insert Menu Item…`, `Insert Stop…`, `Insert Path`, `Insert Conditional…`, `Action Options…`, `Set Options…`, `Playback Options…`, `Save Actions…`, `Load Actions…`, `Replace Actions…`, `Reset Actions`, `Clear All Actions`, `Button Mode`, `Allow Tool Recording`. |
| Action function key | shortcut | `F2`–`F12` (+Ctrl/Cmd, +Shift) | Assigned per action; overrides a colliding command shortcut. |
| `File > Automate > Create Droplet…` | dialog | — | Creates a droplet from an action (`AUTO-002`). |
| `File > Automate > Batch…` | dialog | — | Runs an action over files (`AUTO-003`). |
| `File > Scripts > Script Events Manager…` | dialog | — | Binds events to actions/scripts (`AUTO-004`). |
| `File > Automate > Conditional Mode Change…` | dialog | — | Records a conditional mode-change step. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Action name | string | `Action N` | any Unicode | Stored as a Unicode string in `.atn`. |
| Action set name | string | `Set N` | any Unicode | |
| Function key | enum | None | `None`, `F2`–`F12` | Windows excludes `F1` and `Ctrl+F4`/`Ctrl+F6`. |
| Shift modifier | bool | false | on/off | |
| Cmd/Ctrl modifier | bool | false | on/off | |
| Button colour | enum | None | None, Red, Orange, Yellow, Green, Blue, Violet, Gray | Colour index `0`–`7`. |
| Modal control | tri-state | off | off / on / partial(red) | Cannot be set in Button mode. |
| Command enabled | bool | true | on/off | Unchecked commands are skipped. |
| `withDialog` | bool | per record | on/off | User-toggleable at playback. |
| Dialog options | u8 | `0` | Community-observed: `0` optional, `1` required, `2` none, `3` specific | `3`/`4` undocumented. |
| Playback speed | enum | Accelerated | Accelerated / Step By Step / Pause | |
| Pause duration | seconds | `0` | ≥0 | Only with `Pause For __ Seconds`. |
| Conditional condition | enum | — | e.g. document is square / mode / etc. (list from CS6) | `Then`/`Else` may not both be `None`. |
| Conditional source modes | mode set | All | Bitmap…Lab | `All`/`None` buttons. |
| Conditional target mode | enum | — | Bitmap…Lab | |
| Stop message | string | — | any | `Allow Continue` optional. |

## Algorithms & pipeline

Behavioral parity only; Adobe's exact playback engine is closed. The observable pipeline is:

```text
record:
  UI command → command bus
             → if recording: append ActionStep { event_id, descriptor, dialog_options }
                          (tool recording wraps raw tool input events as steps)

playback(speed):
  for each enabled step in action:
    if step.dialog == modal and displayDialogs allows: prompt / pause for modal tool
    dispatch event_id + descriptor through the command layer
    if speed == StepByStep: repaint document between steps
    if speed == Pause: sleep(pause_seconds)
    if step is Stop: show message (block or Allow-Continue)
    if step is Conditional: evaluate condition → play referenced action (bounded depth)
    if step is InsertPath: set work path
    if playback cancelled / error: stop; error handling per caller (Batch/Droplet)
```

- An action is *not* automatically a single undo step in CS6 (Help advises taking a History snapshot before playback, then restoring it to undo). Kooka Pictura therefore makes "one undo per action run" a **configurable transaction boundary**, not a hard rule (`01-architecture/undo-history`); the scripting `Document.suspendHistory` idiom (`AUTO-010`) is the one-step form.
- `Play` recorded as a step composes actions; recursion must be depth-limited.
- Tool recording stores a time-ordered raw tool-input stream, not just the resulting command, so the recording must be replayable after document changes; this is the least-documented CS6 area (see Open questions).

## Rust module mapping

- `crate::action::model` — `ActionSet { name, expanded, actions }`, `Action { name, function_key, shift, ctrl, color, expanded, steps }`, `ActionStep` enum (`Command { event_id, descriptor, enabled, dialog }`, `Stop { message, allow_continue }`, `MenuCommand { menu_id }`, `InsertPath { path_id }`, `Conditional { cond, then_action, else_action }`, `Play { set, action }`), `ModalState`.
- `crate::action::atn` — big-endian reader/writer for the `.atn`/`.psp` layout and the embedded OSType/descriptor codec. Shares the OSType codec with `crate::script::actions` (no duplicate implementation).
- `crate::action::player` — `Player` that walks `ActionStep`s, calls `crate::command`, honours `PlaybackOptions { speed, pause }`, `ModalState`, `DialogMode`, cancellation, and conditional evaluation.
- `crate::action::registry` — loaded action sets, set/action lookup by name, reserved function-key map, Button-mode colour.
- `crate::action::record` — a `Recorder` subscribing to the command bus event stream; produces `ActionStep::Command` and, under Allow Tool Recording, tool-input steps.
- `crate::command` / `crate::script::actions` — the existing command bus + `ActionDescriptor`/`ActionList`/`ActionReference` types (reused, not redefined). Every action step and every script mutation funnels through this one mutation path.

Boundary types: `ActionDescriptor`/`ActionValue` (shared), `ActionSet`/`Action` (owned, serialisable), `DialogMode` enum, `PlaybackOptions`.

## Qt6 component mapping

Widgets, not QML: the Actions panel is a docked panel over `QMainWindow` and must match CS6's dense list/drag feel.

- `ActionsPanel` (`QDockWidget`) — hosts the tree, buttons, and the panel menu.
- `ActionsTreeModel` (`QAbstractItemModel`) — set → action → step rows; check state = enabled; dialog column = modal state; drag-and-drop reorder/duplicate; `QSortFilterProxyModel` not needed (CS6 has no filter).
- `ActionsPanelMenu` (`QMenu`) — the menu items listed under UI surface.
- `ActionOptionsDialog` / `SetOptionsDialog` — name, function key (with collision check), colour.
- `PlaybackOptionsDialog` — speed + pause seconds.
- `InsertStopDialog` — message + `Allow Continue`.
- `InsertConditionalDialog` — condition combo, Then/Else action combos (shared action-set constraint enforced).
- `ButtonModeView` (`QListView` in icon mode) — flat colour-tagged buttons; falls back to the tree when toggled off.
- `RecordingIndicator` — red record state for the record button.

Drag-and-drop uses `QAbstractItemModel`'s native MIME support; function-key assignment routes through the application `QShortcut` map.

## Data-model impact

- **New persisted types (not in PSD):** `ActionSet`/`Action`/`ActionStep` serialised to `.atn` (Adobe) and/or the project JSON action format (`AUTO-010`); `ActionsPaletteState` for expansion/Button-mode persisted with preferences.
- **In-memory action-descriptor types** (`ActionDescriptor`, `ActionList`, `ActionReference`) are shared with `crate::script` and the plug-in host API (`ARCH-011`) — one definition, one OSType codec.
- **Undo granularity:** configurable — default *per command* (CS6-like; undo by History snapshot), optional *one transaction per action run*. Record shape `HistoryRecord::ActionOp { set, action, step_index }`; pixel payloads reuse the history/tile mechanism.
- **Serialization:** action sets never enter PSD/PSB; XMP untouched. Conditional/Stop/Path steps have no image payload.
- **Function-key map:** a global reserved-shortcut table (preferences) that takes precedence over command shortcuts, matching CS6.

## Edge cases

- **Function-key collision:** an action shortcut overrides the command shortcut with the same keys (documented CS6 behaviour); the options dialog must warn, not block.
- **Non-recordable commands:** recording them directly is impossible; they are only reachable as `Insert Menu Item` steps, which prompt at playback.
- **Button mode:** cannot show steps, cannot set modal controls, cannot exclude commands, cannot edit. Toggling back to list mode must preserve all hidden state.
- **Conditional with both `None`:** invalid; the dialog must reject it. Referenced actions must be in the same set.
- **Recursive `Play`:** action A playing B playing A must be depth-limited and fail with a named error, not hang.
- **Insert Path memory:** complex paths are documented as memory-hungry; playback must not duplicate path geometry per step (a later `Insert Path` replaces the earlier one).
- **Unicode names / locale:** names are Unicode; lookups by name must be locale-stable or expose the raw set/action identity.
- **`.atn` from another version:** version ≠ 16 and unknown command tags must fail with a diagnostic or be preserved opaquely, never silently mis-played.
- **File references in steps:** `Open`/`Save As`/settings-file steps that bake in a path may not resolve on Linux; Batch/Droplet override flags (`AUTO-003`) are the documented escape.
- **Accelerated playback + missing redraw:** dialogs opened by a step must still be honour `displayDialogs`/`DialogMode` (`AUTO-010`); under `NO` a modal step uses recorded values or the command default.
- **Cancellation:** cancelling a play must abort cleanly and (if a transaction was opened) roll it back.
- **Empty action / empty set:** play is a no-op success; save/load round-trips the structure.

## Parity acceptance criteria

1. Given a new action with steps recorded from the UI, playing it reproduces each command's document mutation within the owning command's documented tolerance.
2. Given an action with a modal control on a dialog command, playback pauses and uses the entered values; with the modal control off it uses the recorded values.
3. Given an unchecked (excluded) command, playback skips it and the parent action shows the partial-state indicator.
4. Given `Playback Options = Step By Step`, the image redraws between commands; with `Accelerated` it need not.
5. Given an action with an `Insert Stop` step, playback shows the message and honours `Allow Continue`.
6. Given an `Insert Menu Item` step whose command is not recordable, the command executes at playback and its dialog appears when modal.
7. Given an `Insert Conditional` step, the correct Then/Else action plays for the condition; both-`None` is refused.
8. Given a `.atn` set written by Kooka Pictura and re-read, the set/action/step structure (names, keys, colour, enabled/modal flags, descriptors) round-trips byte-for-byte after decode/encode.
9. Given a `.atn` action set produced by CS6 (community corpus), Kooka Pictura lists its sets/actions and plays commands whose event IDs it supports; unsupported events fail with a named diagnostic.
10. Given a function key assigned to an action, pressing it plays the action and overrides a colliding command shortcut.
11. Given `Button Mode`, clicking a button plays the entire action; switching back to list mode restores step visibility and modal state.
12. Given a `Plays`-recursion cycle, playback aborts with a depth error within a bounded time.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — *Photoshop CS6 Help* pp. 625–644: *About actions and the Actions panel*, *Creating actions* (guidelines, record, path, stop, modal controls, exclude, insert menu item, edit/rerecord), *Playing and managing actions* (play, playback speed, manage, sets), *Adding conditional actions*, *Recording tools in actions | CS6*, *Adding a conditional mode change to an action*; pp. 79–80 What's New (Allow Tool Recording, conditional actions, Contact Sheet II/PDF Presentation). Fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-Scripting-Guide.pdf` — Action Manager chapter (ScriptListener logging, `charIDToTypeID`, `ActionDescriptor`, `executeAction` pattern), notifier/event context. Community mirror; fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-JavaScript-Ref.pdf` — `ActionDescriptor`/`ActionList`/`ActionReference` property and method inventory; `Application.doAction`/`executeAction`; Appendix A (event ID codes). Community mirror; fetched via `curl` + `pdftotext`.
- `https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/` — *Adobe Photoshop File Formats Specification*, "Additional File Formats → Actions": action file types (`8BAC`/`.ATN`), action file header (version 16), action set/action/command layout, descriptor structure and OSType keys, reference structure. Fetched via `curl`.
- `https://raw.githubusercontent.com/LongMarch7/json-photoshop-scripting/master/Documentation/Photoshop-Actions-File-Format/README.md` — community `.atn`/`.psp` format write-up: action-set/action/command fields, Action Manager type encodings, dialog-mode decision table, recording/playback dialog options. Community source.
- Cross-reference `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`) and `docs/09-automation/rust-scripting-replacement.md` (`AUTO-010`) for the shared ActionDescriptor model, command bus, and scripting host.

## Open questions

- **Exact `PlaybackOptions`/`playbackParameters` encoding.** The JS reference exposes `Application.playbackParameters` as an `ActionDescriptor` but not its keys. Resolve by loading CS6 preferences/palette files or community action corpora and decoding the descriptor.
- **Dialog-option value 3 semantics.** Community doc marks values `3`/`4` "undocumented, guessed". Resolve by instrumenting a real CS6 install with a JSON event listener across many commands, or by finding an official spec.
- **Tool-recording fidelity.** Whether CS6 stores raw input events or a parameterised tool command is not documented. Resolve with the ScriptListener/action corpus and a replay-diff harness; if raw-event replay is required, add an `ActionStep::ToolInput` event stream.
- **Undo boundary.** CS6 clearly does not make an action one undo step. Whether Kooka Pictura defaults to CS6-like per-command undo or to a friendlier per-run transaction is a product decision; name it in `01-architecture/undo-history`.
- **Conditional condition list.** The Help lists the feature and one example ("is square") but not the full condition set. Resolve from the CS6 UI or an exhaustive community enumeration.
- **`.atn` interoperability scope.** Whether Kooka Pictura guarantees Adobe-compatible `.atn` read/write or only read, and whether its native action format is `.atn` or JSON (see `ARCH-011` Open questions).
- **Function-key policy on Linux.** CS6 restricts `F1`, `Ctrl+F4`, `Ctrl+F6` on Windows; whether to reproduce those restrictions cross-platform is undecided.
- **Action-set preset provenance.** Shipping predefined actions requires an independently authored action corpus; confirm this is acceptable under `00-overview/licensing-and-provenance`.
