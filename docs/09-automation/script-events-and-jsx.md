# Script Events and JSX

- **Spec ID:** `AUTO-004`
- **Status:** `Draft`
- **Parity tier:** `Core` for the scripting host, `File > Scripts`, the Script Events Manager, and the Action Manager surface; full ExtendScript (E4X, ScriptUI, `Socket`, `ExternalObject`) is `Non-goal (Linux)` and only a documented subset is targeted.
- **New in CS6:** `Changed` — the interpreter stays ECMA-262 3rd ed. + E4X, but the DOM gains tools (tool-preset name), guides arrays, and artboards; the Script Events Manager workflow is unchanged from CS5.
- **Depends on:** `ARCH-011` (plugin-and-scripting-abi), `AUTO-010` (rust-scripting-replacement), `AUTO-005` (extendscript-api-surface), `AUTO-001` (actions), `11-cross-cutting/security-and-sandboxing`, `01-architecture/undo-history`.

## CS6 behavior

Photoshop CS6 supports external automation through COM/ActiveX (Windows: VBScript, JScript), AppleScript/Apple events (macOS), and **JavaScript/ExtendScript** on both platforms. JavaScript is the only cross-platform language and the one Kooka Pictura targets. `ARCH-011` and `AUTO-010` define the engine, sandbox, and DOM in full; this spec covers the observable scripting surface: how scripts are found and executed, the `#target`/`#include` preprocessor, the `app` root, the Action Manager (`doAction`/`executeAction`), ScriptUI dialogs, and the Script Events Manager (`app.notifiers`).

### Script files and discovery

- Scripts are `.jsx` (ExtendScript) or `.js`. `.jsx` runs under ExtendScript; a `.js` double-clicked is interpreted by the OS and does not necessarily launch Photoshop.
- `File > Scripts > <name>` lists `.js`/`.jsx` files placed in `Presets/Scripts`, by base name. They appear only after the application restarts.
- `File > Scripts > Browse…` runs a script from any location.
- **Startup scripts** run at launch; a shared startup script typically guards itself (e.g. `if (BridgeTalk.appName == "photoshop") { … }`).
- CS6 built-in scripts surfaced in the menu include `Image Processor`, `Delete All Empty Layers`, `Export Layers To Files`, `Load Files into Stack`, `Layer Comps to Files`, and `Statistics`.

### Preprocessor directives

ExtendScript files are preprocessed before evaluation. The directives that matter:

- `#target photoshop` — declares the host application, so a `.jsx` can be launched from anywhere. In a single-host application the Kooka Pictura shim treats it as a no-op guard.
- `#targetengine <name>` — selects a named persistent engine (ExtendScript Toolkit convention).
- `#include "file.jsx"` / `#includepath "dir"` — textual inclusion resolved at preprocess time. Kooka Pictura resolves these through the per-script module allowlist (`AUTO-010`), rejecting traversal.
- `#script "…"` / `#strict` — script-name metadata and strict parsing.

### The `app` root and Action Manager

`app` (`Application`) is the object-model root. The scripting-relevant members:

- `app.doAction(name, actionSet)` — plays a named action from the Actions panel (`AUTO-001`).
- `app.executeAction(eventID, descriptor[, displayDialogs])` — plays a raw Action Manager event; returns an `ActionDescriptor`. `app.executeActionGet(reference)` reads state.
- `app.charIDToTypeID("Embs")` / `app.typeIDToCharID(id)` and `app.stringIDToTypeID(str)` / `app.typeIDToStringID(id)` — convert between four-character codes, runtime IDs, and string IDs.
- `app.displayDialogs` (`DialogModes.ALL` | `ERROR` | `NO`) and `app.playbackDisplayDialogs` — control whether dialogs appear during script/action playback.
- `app.runMenuItem(menuID)`, `app.showColorPicker()`, `app.beep()`, `app.refresh()`, progress APIs, and the preferences/`Documents` roots (`AUTO-005`).

The canonical low-level pattern, produced by **ScriptListener**, is:

```js
var id19 = charIDToTypeID( "Embs" );
var desc4 = new ActionDescriptor();
desc4.putInteger( charIDToTypeID( "Angl" ), 135 );
desc4.putInteger( charIDToTypeID( "Hght" ), 3 );
desc4.putInteger( charIDToTypeID( "Amnt" ), 100 );
executeAction( id19, desc4, DialogModes.NO );
```

ScriptListener is an automation plug-in (`ScriptListener.8li`) copied into `Plug-Ins/Automate`; it appends `ScriptingListenerJS.log` / `ScriptingListenerVB.log` on the desktop. It is the documented way to discover event IDs and descriptor keys for behavior not covered by the typed DOM. Adobe notes there is no AppleScript interface to the Action Manager; AppleScript users run JavaScript Action Manager code.

### ScriptUI dialogs

ScriptUI lets scripts build dialogs and controls (`Window`, panels, buttons, edit fields, lists, tabs, sliders, radio/checkbox, tree, drawing, layout managers, event callbacks) from JavaScript. Kooka Pictura's position (`AUTO-010`): a minimal Qt-backed shim for dialogs, or a documented rewrite; full ScriptUI is not promised.

### Script Events Manager and notifiers

`File > Scripts > Script Events Manager` maps an application event to a script or an action:

1. Enable `Enable Events To Run Scripts/Actions`.
2. Choose an event from the Photoshop Event menu.
3. Choose `Script` (with a sample/`Browse…`) or `Action` (set + action).
4. `Add`; the mapping is listed. `Remove` deletes an entry; disabling the checkbox keeps entries but stops firing.

The scripting equivalent is `app.notifiersEnabled` and `app.notifiers`:

```js
app.notifiersEnabled = true;
var eventFile = new File(app.path + "/Presets/Scripts/Event Scripts Only/Welcome.jsx");
app.notifiers.add("Opn ", eventFile);   // "Opn " = Open Document event ID
```

- Events are identified by a four-character ID (`'Opn '`, `'Cls '`, `'Sav '`, …) or a string ID; event IDs come from Appendix A of the JavaScript reference or from ScriptListener.
- Some events need an additional **class ID** (e.g. `New` applies to `Document`, `ArtLayer`, `Channel`).
- **Important CS6 caveat:** notifications generally do **not** fire for events that occur *inside a script*, because those events are wrapped in an `"AdobeScriptAutomation Scripts"` event. Notifiers observe user/committed events, not a script's own mutations.

### How Photoshop executes a script

```text
File > Scripts (or Browse, or event notifier, or startup)
        │
        ▼
 preprocess (#target / #include / #strict)  →  source
        │
        ▼
 ExtendScript engine (ES3 + E4X + Adobe extensions)
        │  app.* calls
        ▼
 typed DOM  ──►  command layer  ──►  document mutations
        └── executeAction(eventID, descriptor) ──► same command layer
        │
        ▼
 events committed → notifier table → (deferred) fire matching script/action
```

Scripts run in the host's JavaScript engine; there is no COM/AppleScript bridge to reproduce on Linux. All mutations route through one command/action layer shared with the UI and native plug-ins (`ARCH-011`), which is what makes undo, dialog mode, and notifier ordering consistent.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `File > Scripts > <script>` | menu | — | Installed scripts from `Presets/Scripts`, by base name; appears after relaunch. |
| `File > Scripts > Browse…` | dialog | — | Run a script from any location; `.js`/`.jsx` filter. |
| `File > Scripts > Script Events Manager…` | dialog | — | Event ↔ script/action mapping (parity). |
| `File > Scripts > <built-in>` | menu | — | `Image Processor`, `Statistics`, etc. |
| Script console / error panel (new) | panel | — | Output, tracebacks, Run/Stop (`AUTO-010`). |
| Script security prompt (new) | dialog | — | Per-script filesystem/network grant (`11-cross-cutting/security-and-sandboxing`). |
| Status bar / progress | widget | — | `doProgress`/`doProgressTask`/`changeProgressText` bridge. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| `displayDialogs` | enum | `ALL` | `ALL` (`1`), `ERROR` (`2`), `NO` (`3`) | Mirrors CS6. |
| `playbackDisplayDialogs` | enum | `ALL` | as above | Dialog mode during action playback. |
| `notifiersEnabled` | bool | false | on/off | Global notifier switch. |
| Notifier event | ID | — | four-char ID or string ID | Appendix A / ScriptListener. |
| Notifier class | ID | — | class ID for ambiguous events (`New`, …) | Optional. |
| Script timeout | duration | `30 s` | `0` = unlimited | rquickjs interrupt handler (`AUTO-010`). |
| Memory / stack limits | bytes | `64 MiB` / `256 KiB` | `0` = unlimited | `AUTO-010`. |
| Module/include allowlist | path list | empty (deny) | per script | `#include` resolution. |
| Engine | enum | QuickJS | QuickJS \| Lua \| Rhai | Proposed primary (`AUTO-010`). |

## Algorithms & pipeline

- **Preprocess:** strip/handle `#target`, `#targetengine`, `#include`, `#includepath`, `#script`, `#strict`; resolve includes textually within the allowlist; produce an error naming the offending directive otherwise. `#target photoshop` is accepted as a single-host no-op.
- **Engine init / limits / DOM binding / execution / transaction:** exactly as specified in `AUTO-010` — one `Rust` script thread owning `rquickjs::Runtime`/`Context`, memory/stack/interrupt limits, DOM classes calling the command bus, and **one host undo transaction per top-level script run** (the `Document.suspendHistory` idiom).
- **Action Manager:** `charIDToTypeID`/`stringIDToTypeID` and their inverses are backed by a checked-in four-char ↔ string table (independent-creation; see licensing). `executeAction` looks up an event registry that reuses the same command objects the UI builds; unknown event IDs error rather than no-op.
- **Notifiers:** the command bus publishes committed events; the notifier table matches event (and class) IDs to a script or action and fires it **after the current transaction commits**, never re-entrantly, with a depth limit. Events generated *inside* a script are tagged as script-internal and (matching CS6) do not trigger notifiers.
- **ScriptUI (proposed shim):** a small `Window`/control vocabulary composed over Qt dialogs on the UI thread, with the script thread parked while the dialog is modal.

## Rust module mapping

Refines the `crate::script::*` layout of `ARCH-011` / `AUTO-010` (reused, not redefined):

- `crate::script::engine` — `ScriptHost` on a dedicated thread; limits and timeout.
- `crate::script::preprocess` — `#target`/`#include`/`#includepath`/`#script`/`#strict` handling, include resolution, diagnostics.
- `crate::script::dom` — `Application`, `Document`, `Layer`, … as QuickJS classes over `crate::command`.
- `crate::script::actions` — `ActionDescriptor`/`ActionList`/`ActionReference`, `execute_action`, `do_action`, id mapping.
- `crate::script::notifiers` — `NotifierBinding { event_id, class_id?, target: Script(path) | Action(set, name), enabled }`; subscription table fed by the command event stream.
- `crate::script::compat` — `File`/`Folder`/`alert`/`prompt`, name aliases, E4X detection.
- `crate::script::sandbox` / `crate::script::registry` — permissions and installed-script discovery (`AUTO-010`).

Boundary types are plain values; no Rust borrows escape a call.

## Qt6 component mapping

- `ScriptsMenu` — model-driven from `ScriptRegistryModel`; includes `Browse…` and `Script Events Manager…`.
- `ScriptEventsManagerDialog` — event ↔ script/action editor backed by `NotifierBindingModel`; enable checkbox, Add/Remove.
- `ScriptConsoleDock` — `QPlainTextEdit` output + Run/Stop `QToolBar`, error highlighting, timings.
- `ScriptSecurityDialog` — per-script filesystem/network permission prompt (`ScriptPermissionModel`).
- `ProgressProxy` — `QProgressDialog` bridging `doProgress(SubTask)`/`changeProgressText`; Cancel → `ScriptError::Cancelled`.
- `ScriptUI shim` (optional) — `QDialog`/`QFormLayout` built from a small widget vocabulary; QML only if later justified.

Widgets, not QML: console and managers are document-modal/docked utilities over `QMainWindow`.

## Data-model impact

- **New in-memory types:** `ScriptLimits`, `ScriptPermission`, `NotifierBinding` (`ARCH-011`/`AUTO-010`).
- **New persisted types (preferences, not PSD):** `ScriptEventBinding { event_id, class_id?, target, enabled }` — mirrors CS6 notifiers and the Script Events Manager list; `ScriptRegistryEntry { path, display_name, enabled }`.
- **Undo granularity:** one top-level script run = one history state labelled with the script name; nested `executeAction`s fold into it (`Document.suspendHistory` semantics). Undo record `HistoryRecord::ScriptOp { script_path, command_ids }`.
- **Serialization:** scripts and bindings are never written into PSD/PSB; XMP untouched. Action descriptors serialize to the action format when exported as actions.
- **Events:** notifier firing uses the committed-command event stream; script-internal events are tagged and suppressed for notifier matching (CS6 parity).

## Edge cases

- **E4X / operator overloading:** unsupported; detected and reported with a diagnostic naming the construct, never a bare syntax error (`AUTO-010`).
- **`File`/`Folder`/`Socket`/`ExternalObject`:** sandboxed replacements or unsupported; default-deny filesystem/network unless granted; symlink/`..` traversal rejected.
- **`#include` cycles / missing include:** fail with a named file/line diagnostic; includes resolved only within the allowlist.
- **`#target` other than photoshop:** accepted as a no-op (single host) but logged at debug.
- **Infinite loop / stack / memory bomb:** interrupt handler + stack/memory limits; trip aborts the transaction, rolls back, and surfaces a traceback within the timeout.
- **Notifier recursion / reentrancy:** notifier scripts run after commit, never reentrantly; a notifier that triggers its own event is depth-limited.
- **Script-internal events:** must not fire notifiers (CS6 `"AdobeScriptAutomation Scripts"` wrapping).
- **Dialog mode:** `displayDialogs = NO` suppresses UI (batch/automation); a modal ScriptUI dialog under `NO` returns default/cancel rather than blocking.
- **Unimplemented DOM method/property:** throws a typed `NotImplementedError` naming the CS6 API, so gaps are visible.
- **`.js` vs `.jsx`:** `.js` not launched by the application should not be assumed to run under the host engine; only `.jsx` (and `.js` invoked through `File > Scripts`) is guaranteed.
- **Startup scripts:** guarded startup scripts must not crash launch; a failing startup script is logged and skipped.
- **Encoding/locale:** sources are UTF-8; string IDs from non-Latin locales resolve through the id table or fail loudly.
- **Permissions on notifiers:** auto-running scripts on events is an attack surface; binding creation requires user confirmation and stored bindings are reviewed in the manager.

## Parity acceptance criteria

1. Given a `.jsx` in the scripts directory, restarting lists it under `File > Scripts` by base name; choosing it runs it.
2. Given `#target photoshop` at the top of a script, it runs without a "no target application" error.
3. Given `#include "lib.jsx"` where `lib.jsx` is inside the allowlist, the included code runs; when outside, the script fails with a diagnostic naming the path.
4. Given `app.activeDocument.activeLayer.opacity = 50`, the active layer opacity becomes 50 and exactly one history state named after the script is created.
5. Given the ScriptListener pattern (`charIDToTypeID` + `ActionDescriptor.putInteger` + `executeAction`), the resulting document mutation matches the equivalent UI command within tolerance; an unknown event ID errors.
6. Given `app.displayDialogs = DialogModes.NO`, a script that would open a modal dialog completes without UI and without blocking.
7. Given `app.notifiers.add("Opn ", file)` with notifications enabled, opening a document outside a script runs the script; an `Open` performed *inside* a script does not (CS6 wrapping semantics).
8. Given a Script Events Manager entry mapping an event to an action, the action plays when the event occurs; `Remove`/disable stops it.
9. Given a script exceeding the timeout, the host interrupts within ±1 s, rolls back, and adds no history state.
10. Given an E4X literal, the script fails with an explicit "E4X unsupported" diagnostic naming the construct.
11. Given a script calling an unimplemented CS6 DOM method, the error names that method.
12. Given a script that triggers a notifier whose script triggers another notifier, firing terminates within the depth limit with an error rather than hanging.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — *Scripting* chapter: COM/AppleScript/JavaScript support, `File > Scripts` list and `Browse`, `.js`/`.jsx` in `Presets/Scripts`, Script Events Manager (enable, event, script/action, Add/Remove). Fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-Scripting-Guide.pdf` — *Photoshop Scripting Basics* (DOM containment hierarchy, `#target`, `Presets/Scripts`); *Scripting Photoshop* (targeting the Application, dialogs, Notifier objects and `app.notifiers.add("Opn ", file)`); *Action Manager* (ScriptListener install, `ScriptingListenerJS.log`/`ScriptingListenerVB.log`, `charIDToTypeID`/`ActionDescriptor`/`executeAction`). Community mirror; fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-JavaScript-Ref.pdf` — `Application.{doAction, executeAction, executeActionGet, charIDToTypeID, typeIDToCharID, stringIDToTypeID, typeIDToStringID, runMenuItem, displayDialogs, notifiers, notifiersEnabled, playbackDisplayDialogs}`, `Notifier`/`Notifiers`, Appendix A event ID codes. Community mirror; fetched via `curl` + `pdftotext`.
- `https://extendscript.docsforadobe.dev/introduction/extendscript-overview/` — ExtendScript language/engine overview for Adobe applications. Fetched via `curl`.
- `https://extendscript.docsforadobe.dev/extendscript-tools-features/preprocessor-directives/` — `#include`, `#includepath`, `#script`, `#strict`, `#target`, `#targetengine`. Fetched via `curl`.
- Cross-reference `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`) and `docs/09-automation/rust-scripting-replacement.md` (`AUTO-010`) for the engine, sandbox, migration phases, and the underlying Sources list.

## Open questions

- **Notifier event table completeness.** Appendix A lists many events but is not exhaustive; resolve by mining ScriptListener output and community corpora, and by deciding which events the host can emit.
- **Script-internal event wrapping.** The exact rule for "events inside a script don't fire notifiers" is documented only as the `"AdobeScriptAutomation Scripts"` behaviour; reproduce or refine and verify against CS6.
- **ScriptUI scope.** Minimal Qt dialog shim vs fuller ScriptUI vs rewrite-only (`AUTO-010` Open questions). Determines how many real `.jsx` run unchanged.
- **`#include` semantics.** Whether ExtendScript's include is purely textual and how duplicate includes/hoisting behave; resolve against ExtendScript documentation/tests.
- **`#targetengine`.** Persistent named engines are an ExtendScript Toolkit feature with no obvious single-host equivalent; whether to accept-and-ignore or reject is open.
- **Action Manager event coverage.** Which CS6 event IDs Kooka Pictura implements, and the fallback for the rest, is not enumerated; resolve against the event-ID appendix and real actions.
- **Security of event bindings.** Persisted auto-run bindings are an attack surface; the confirmation/review policy belongs to `11-cross-cutting/security-and-sandboxing`.
- **Script discovery freshness.** CS6 requires a relaunch for new `Presets/Scripts` entries; whether Kooka Pictura keeps that or watches the directory is a UX decision.
- **Deprecated/CS5-era methods.** Whether to shim `makeContactSheet`/`makePDFPresentation`/`makePhotoGallery`/`makePicturePackage` (`AUTO-003`).
