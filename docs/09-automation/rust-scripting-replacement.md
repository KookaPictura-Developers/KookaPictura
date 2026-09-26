# Rust Scripting Replacement (Scripting Engine)

- **Spec ID:** `AUTO-010`
- **Status:** `Draft`
- **Parity tier:** `Core` (scripting host); full ExtendScript/E4X compatibility is `Non-goal (Linux)` and only a documented subset is targeted.
- **New in CS6:** `Changed` — the ExtendScript interpreter is unchanged (ECMA-262 3rd ed. + E4X). CS6 adds DOM surface: the tool name associated with a tool preset is scriptable, and a document returns an array of guides from the scripting SDK.
- **Depends on:** `ARCH-011` (plugin-and-scripting-abi), `09-automation/extendscript-api-surface`, `09-automation/actions`, `11-cross-cutting/security-and-sandboxing`, `01-architecture/undo-history`, `01-architecture/document-model`.

## CS6 behavior

Photoshop CS6 exposes external automation through scripting. On Windows, COM/ActiveX automation (VBScript, JScript) is available; on macOS, AppleScript/Apple events. JavaScript/ExtendScript is the only cross-platform language and the one on which this spec focuses (`ARCH-011` describes the interpreter and object model in detail; this spec defines the replacement engine, the object model to expose, sandboxing, the compatibility shim, and the action-descriptor layer).

Observable CS6 behavior this spec must preserve:

- Scripts are `.jsx` (ExtendScript) or `.js` files. `.jsx` runs under ExtendScript; a `.js` launched by double-click is interpreted by the OS (JScript on Windows) and does not necessarily launch Photoshop.
- Scripts placed in `Presets/Scripts` appear, by base name, at the top level of `File > Scripts` (not hierarchical, even for sub-folders). They appear only after a relaunch. `File > Scripts > Browse...` runs a script from any location.
- On startup Photoshop executes every `.jsx` in the CS6 startup folders. A shared startup script guards itself with `if (BridgeTalk.appName == "photoshop") { ... }`.
- The object model is rooted at the global `app` (`Application`) and contains `Document`/`Documents`, `Layer`/`ArtLayer`/`LayerSet` and their collections, `Channel`, `Selection`, `PathItem`, `HistoryState`, `LayerComp`, `TextItem`, and the ActionManager types `ActionDescriptor`, `ActionList`, `ActionReference`.
- The ActionManager is the scripting escape hatch for anything the typed DOM does not cover. `ScriptListener` records user operations as `ActionDescriptor` code; the canonical pattern is `charIDToTypeID` + `new ActionDescriptor()` + `putInteger(...)` + `app.executeAction(id, desc, DialogModes.NO)`.
- `Document.suspendHistory` wraps a whole script into a single history state (one undo for the script), which is the CS6 idiom the Kooka Pictura equivalent must reproduce.
- `File > Scripts > Script Events Manager` maps application events to a script or an action (`Notifier`s).

Kooka Pictura does not ship ExtendScript and does not promise binary/behavioural equivalence for arbitrary `.jsx`. It ships a new embedded engine, a Kooka Pictura DOM that follows CS6 naming, and a compatibility shim that covers the common script surface.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `File > Scripts > <script name>` | menu | — | One entry per installed script; populated from the scripts directory. |
| `File > Scripts > Browse...` | menu | — | File dialog; `.js`/`.jsx` filters. |
| `File > Scripts > Script Events Manager...` | dialog | — | Event ↔ script/action mapping (parity with CS6; see `ARCH-011`). |
| `File > Scripts > <built-in>` | menu | — | CS6 built-ins: `Image Processor`, `Delete All Empty Layers`, `Export Layers To Files`, `Load Files into Stack`, `Layer Comps to Files`, `Statistics` (script-provided). |
| Script console / error panel (new) | panel | — | Output, errors, timings; Run/Stop (ExtendScript Toolkit equivalent). |
| Script security prompt (new) | dialog | — | Per-script filesystem/network grant; see `11-cross-cutting/security-and-sandboxing`. |
| Status bar / progress | widget | — | `doProgress`/`doProgressTask` bridge. |

## Parameters & ranges

No user-adjustable image parameters. The controls are engine/host limits.

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Engine | enum | QuickJS | QuickJS \| Lua \| Rhai | Proposed primary is QuickJS; see Algorithms. |
| Script timeout | duration | `30 s` | `0` = unlimited | `rquickjs` interrupt handler / Lua hook. |
| Memory limit | bytes | `64 MiB` | `0` = unlimited | `Runtime::set_memory_limit`; no-op with a custom allocator. |
| Stack limit | bytes | `256 KiB` | — | `Runtime::set_max_stack_size`; rquickjs default is `256 * 1024`. |
| Module allowlist | path list | empty (deny all) | per script | `Runtime::set_loader` (feature `loader`). |
| Output capture | bool | on | on/off | Console + return value. |
| `displayDialogs` | enum | `ALL` | `ALL`, `ERROR`, `NO` | Mirrors CS6 `app.displayDialogs`. |
| Action-descriptor mode | enum | typed | typed \| raw | Typed DOM vs `executeAction`; both available. |

## Algorithms & pipeline

### Engine selection

Compared from the crates' own documentation (versions current at time of writing: `rquickjs` 0.13.0, `mlua` 0.12.1):

| Engine | Rust crate | Language | Sandbox primitives (documented) | Fit |
|---|---|---|---|---|
| QuickJS | `rquickjs` | JavaScript (ES2025-era) | `set_memory_limit`, `set_max_stack_size` (default `256*1024`), `set_interrupt_handler` (return `true` = uncatchable exception), `set_loader` (`Resolver`/`Loader`, feature `loader`), custom allocator, `run_gc`/`memory_usage`; `Send`/`Sync` only with feature `parallel` | Closest to ExtendScript (both JS); shortest migration; native JSON. |
| Lua | `mlua` | Lua 5.1–5.5 / LuaJIT / Luau | `StdLib` subsetting, `UserData` (only what you register is reachable), debug hooks (`HookTriggers`) and `VmState`, `!Send` by default (`send` feature), `async` feature | Fast and mature; DOM ergonomics/tooling weaker for JS authors; no E4X/DOM parity. |
| Rhai | `rhai` | Rhai (JS-like) | Sandboxed by design; an immutable `Engine` cannot mutate state and is re-entrant; documents DoS vectors (string/array/BLOB/object growth, variable/function creation, tight loops, deep recursion, self-referencing modules) and a "Don't Panic" guarantee; `unchecked` disables all safety checks | Best safety story and simplest embedding; not JavaScript; no E4X; ecosystem smaller. |

**Recommendation (provisional):** primary engine **QuickJS via `rquickjs`**, because the existing corpus and third-party scripts are JavaScript, QuickJS is small and embeddable without ICU, and it exposes explicit memory/stack/interrupt controls plus controlled module loading. Rhai is the documented fallback for users who prefer a non-JS, sandbox-first language. Lua is not preferred.

### Execution model

```text
UI / command bus thread
        │  run(script_source, permissions)
        ▼
ScriptHost thread (owns one rquickjs::Runtime + Context)
  1. build Runtime; set_memory_limit, set_max_stack_size
  2. set_interrupt_handler: closure returns true once wall-clock > timeout
  3. set_loader: resolve module ids against the per-script allowlist
  4. install DOM globals (app, alert/prompt, $, File/Folder shim)
  5. begin one host undo transaction (label = script name)
  6. evaluate; pump pending jobs (promises) until done
  7. commit transaction on success; abort+rollback on throw/timeout
        │
        ▼
   CommandBus / ActionDescriptor layer  → document mutations (single undo)
```

All mutations from scripts go through the same command/action layer as the UI and native plug-ins (`ARCH-011`, `crate::command`). That gives scripting, actions, and plug-ins one undo path and one place to enforce dialog mode.

### Object model to expose (proposed)

A pragmatic subset, named after CS6 classes:

| CS6 object | Kooka Pictura surface | Backing |
|---|---|---|
| `Application` (`app`) | `activeDocument`, `documents`, `foregroundColor`, `backgroundColor`, `currentTool`, `preferences`, `version`, `displayDialogs`, `notifiers` | `crate::command` + preferences store |
| `Document`/`Documents` | canvas dims, `mode`, `resolution`, `layers`, `channels`, `pathItems`, `historyStates`, `layerComps`, `selection`, `save`/`saveAs`/`exportDocument`, `resizeImage`/`resizeCanvas`, `suspendHistory` | `crate::document` |
| `Layer`/`ArtLayer`/`LayerSet` + collections | `name`, `opacity`, `blendMode`, `visible`, `bounds`, masks, styles, `textItem` | `crate::document`, commands |
| `Channel`, `Selection`, `PathItem`, `HistoryState`, `LayerComp`, `TextItem` | typed wrappers | respective core modules |
| `ActionDescriptor`/`ActionList`/`ActionReference` | first-class map/list types | `crate::action` |
| `Notifier`/`Notifiers` | event bindings | `crate::script::notifiers` |
| `Preferences`, save/open option classes | typed option structs | `crate::preferences` |

`app.executeAction`, `app.doAction`, `app.executeActionGet`, `charIDToTypeID`/`typeIDToCharID`, `stringIDToTypeID`/`typeIDToStringID`, `runMenuItem`, `showColorPicker`, `purge`, `refresh`, `beep` are exposed. Deprecated or platform-only methods are absent unless listed.

### Partial ExtendScript-compatibility shim

The shim is deliberately partial, and the boundary is stated up front (`ARCH-011`):

| ExtendScript feature | Kooka Pictura decision |
|---|---|
| ES3 syntax, `var`, functions, prototypes, `#target`, `#include`, `#includepath`, `#script`, `#strict` | Supported by a preprocessor + JS engine. `#target photoshop` is a no-op guard (single-host); `#include` resolves through the module allowlist. |
| E4X `XML` literals, operator overloading | **Not supported.** Detected and reported with a diagnostic; needs rewrite or a third-party transpiler. |
| `File` / `Folder` | Sandboxed replacements (allowlisted roots, `..`/symlink traversal rejected). |
| `Socket`, `ExternalObject` | **Not supported** (no native external object loading). |
| ScriptUI (`Window`, controls) | Minimal Qt-backed shim (dialogs only) or documented rewrite; see Open questions. |
| `alert`/`prompt`/`confirm` | Mapped to Qt dialogs; suppressed to defaults under `displayDialogs = NO`. |
| `BridgeTalk` | Absent (single application). |
| DOM typed objects | Mapped to the Kooka Pictura DOM subset above via the command layer. |
| Unimplemented method/property | Throws a typed `NotImplementedError` with the CS6 name, so scripts fail loudly, not silently. |

A script converter (Phase 3 below) rewrites E4X-free ES3 to the shim where mechanical, and flags the rest.

### Action-descriptor equivalent

The CS6 ActionManager is reproduced as a typed, serialisable `ActionDescriptor`:

```text
ActionKey   = U32 (four-char) | String (stringID)
ActionValue = Integer(i32) | Double(f64) | Boolean | String(OpStr)
            | UnitDouble { unit: Unit, value: f64 }
            | Enumerated { type_id: ActionKey, enum_id: ActionKey }
            | Reference(ActionReference) | List(ActionList)
            | Descriptor(ActionDescriptor) | Null
ActionList      = Vec<ActionValue>
ActionReference = { property?: ActionKey, class?: ActionKey, name?: String,
                    index?: i32, identifier?: i64, offset?: i32 }
```

- `charIDToTypeID`/`stringIDToTypeID` and inverses are backed by a four-char ↔ string table generated from a checked-in mapping file (not copied from Adobe assets; see legal note).
- `executeAction(event_id, descriptor, dialog_mode)` looks up the event in a registry that reuses the exact same command objects the UI builds. Unknown event ids return an error rather than a silent no-op.
- Descriptors serialise to the project's action format (JSON) and, if in scope, import/export Adobe `.atn` (see Open questions).

### Migration path

1. Phase 1 — engine + DOM subset: open/save, layer properties, `doAction`, selector filters; one undo per script.
2. Phase 2 — ActionDescriptors (`executeAction`, `charIDToTypeID`), `Notifier`s, Script Events Manager.
3. Phase 3 — compatibility shim (`#target`/`#include`, `File`/`Folder`, `alert`), script converter; E4X diagnostics.
4. Phase 4 — ScriptUI shim over Qt; full `File`/`Folder` sandbox.
5. Phase 5 — Rhai documented as the sandbox-first alternative engine behind the same DOM.

## Rust module mapping

These refine the `crate::script::*` layout named in `ARCH-011`:

- `crate::script::engine` — `ScriptHost` owning `rquickjs::Runtime`/`Context` on a dedicated thread; applies limits; owns job pump and timeout.
- `crate::script::limits` — `ScriptLimits { timeout, memory, stack, module_roots }`; translates to rquickjs runtime calls; surfaces a uniform `ScriptError::LimitExceeded`.
- `crate::script::dom` — QuickJS classes (`Application`, `Document`, `Layer`, ...) as `JsClass` wrappers that call `crate::command`.
- `crate::script::actions` — `ActionDescriptor`/`ActionList`/`ActionReference` (shared with `crate::action`), `execute_action`, id-mapping tables.
- `crate::script::compat` — preprocessor directives, name aliases, `File`/`Folder`, `alert`/`prompt`, E4X detection.
- `crate::script::sandbox` — permission model, allowlist resolution, output capture.
- `crate::script::notifiers` — event subscription table fed by the command bus event stream.
- `crate::script::registry` — installed-script discovery and `File > Scripts` menu model.

Boundary types crossing into JS are plain values (numbers, strings, `ActionValue`); no Rust borrows escape a call.

## Qt6 component mapping

- `ScriptConsoleDock` — `QPlainTextEdit` output + Run/Stop `QToolBar`; error highlighting; timing readout.
- `ScriptsMenu` — built into `QMenuBar`, model-driven from `ScriptRegistryModel`; includes `Browse...`, `Script Events Manager...`.
- `ScriptEventsManagerDialog` — event ↔ script/action editor (CS6 parity), backed by `NotifierBindingModel`.
- `ScriptSecurityDialog` — per-script filesystem/network permission prompt; `ScriptPermissionModel`.
- `ProgressProxy` — `QProgressDialog` bridging `doProgress(SubTask)`; Cancel maps to `OP_ERR_CANCELLED`/`ScriptError::Cancelled`.
- `ScriptUI shim` (if built) — `QDialog`/`QFormLayout` composed from a small widget vocabulary; QML only if layout flexibility is later required.

Widgets (not QML) are proposed for console/dialogs: they are document-modal utilities over `QMainWindow` that must match the docked-panel feel of CS6.

## Data-model impact

- **New in-memory types:** `ActionDescriptor`, `ActionList`, `ActionReference` (shared with `crate::action`); `ScriptLimits`; `ScriptPermission`.
- **New persisted types (preferences/database, not PSD):** `ScriptEventBinding { event_id, target: Script(path) | Action(set, name), enabled }` (mirrors CS6 notifiers); `ScriptRegistryEntry { path, display_name, enabled }`.
- **Undo granularity:** one top-level script invocation = one history state, labelled with the script name (`Document.suspendHistory` semantics). Nested `executeAction`s and progress sub-tasks fold into the parent transaction. A script that itself calls a `suspendHistory`-like scope resets the boundary.
- **Undo record shape:** `HistoryRecord::ScriptOp { script_path, command_ids }`. Pixel payloads reuse the existing history/tile mechanism; no script-specific pixel store.
- **Serialization:** scripts and their metadata are **not** written into PSD/PSB. ActionDescriptors serialise to the action XML/JSON representation when exported. XMP is untouched.

## Edge cases

- **Infinite loop / stack blow-out / memory bomb:** interrupt handler, stack limit, memory limit; on trip, abort and roll back the script's transaction and surface a traceback. Rhai's documented DoS list is the checklist if Rhai is adopted.
- **Memory limit vs custom allocator:** `set_memory_limit` is documented as a no-op when a custom allocator is in use; if Kooka Pictura adopts the `rust-alloc`/`allocator` feature, the allocator itself must enforce the cap.
- **Promises/jobs:** a script that schedules jobs and returns must still be run to completion; the host pumps `is_job_pending`/`execute_pending_job` until drained or timeout.
- **`Send`/`Sync`:** rquickjs `Runtime` is `Send`/`Sync` only with the `parallel` feature; otherwise the runtime is confined to the script thread. The host must not share it.
- **Dialog mode:** `displayDialogs = NO` suppresses modal UI for batch processing; a script or dialog requesting a modal under `NO` gets a default or `OP_ERR_CANCELLED`.
- **Reentrancy:** a script that triggers an event which fires a notifier that runs a script must be depth-limited; notifier scripts are executed after the current transaction commits, never re-entrantly.
- **Unimplemented DOM access:** must throw `NotImplementedError` naming the CS6 API, so migration gaps are visible instead of silently wrong.
- **File/module access:** default-deny; symlink and `..` traversal rejected; no `ExternalObject`.
- **Locale/encoding:** script source is UTF-8; string ids from non-Latin locales resolve through the id table or fail loudly.
- **8/16/32-bit and CMYK/Lab documents:** the DOM reports values in document-native units; colour-conversion helpers are explicit, not implicit.

## Parity acceptance criteria

1. Given a `.jsx` in the scripts directory, it appears under `File > Scripts` by base name after relaunch, and running it executes under the chosen engine.
2. Given `#target photoshop` in a script, it runs without a "no target application" error (single-host no-op).
3. Given `app.activeDocument.activeLayer.opacity = 50`, the active layer opacity becomes 50 and exactly one history state named after the script is created.
4. Given the ScriptListener pattern `charIDToTypeID("Embs")` + `ActionDescriptor` + `executeAction`, applying an Emboss-equivalent matches the UI command within the per-command tolerance documented by the filter spec.
5. Given `app.displayDialogs = DialogModes.NO` (or the shim equivalent), a script that would open a modal completes without UI and without blocking.
6. Given a script that runs longer than the configured timeout, the host interrupts it within ±1 s, rolls back its transaction, and adds no history state.
7. Given a script that reads a file outside the granted roots, the read fails and the denial is logged; inside the granted roots it succeeds.
8. Given an E4X `<xml/>` literal, the script fails with an explicit "E4X unsupported" diagnostic naming the offending construct, not a syntax error with no context.
9. Given a script calling an unimplemented CS6 DOM method, the error names that method.
10. Given the same script run twice, the document pixel hash after run 2 equals run 1 (determinism of the command path), modulo documented non-deterministic filters.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — Scripting chapter (COM/AppleScript/JavaScript, `.js`/`.jsx`, `File > Scripts` list, startup scripts, `Script Events Manager`); What's New scripting notes (tool-preset name, guides array). Fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-Scripting-Guide.pdf` — *Adobe Photoshop CS6 Scripting Guide*: script installation, startup folders, `#target`, ActionManager chapter (`charIDToTypeID`, `ActionDescriptor`, `executeAction`), JavaScript/VBScript ActionManager examples. Community mirror; fetched via `curl` + `pdftotext`.
- `https://docs.rs/rquickjs/latest/rquickjs/runtime/struct.Runtime.html` — `Runtime::{new, new_with_alloc}`, `set_memory_limit` (0 = unlimited; no-op with custom allocator), `set_max_stack_size` (default `256*1024`), `set_interrupt_handler`, `set_loader`, `run_gc`, `memory_usage`, `is_job_pending`/`execute_pending_job`, `Send`/`Sync` behind `parallel`.
- `https://docs.rs/mlua/latest/mlua/` — Lua 5.1–5.5/LuaJIT/Luau, `Lua`, `UserData`, `StdLib`, debug `HookTriggers`, `VmState`, `!Send` default with `send` feature, async support.
- `https://rhai.rs/book/safety/index.html` — Rhai DoS vectors, "Don't Panic" guarantee, `unchecked` disabling safety.
- `https://rhai.rs/book/safety/sandbox.html` — Rhai sandboxing, immutable `Engine`, re-entrancy.
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/Document.html` — CS6 `Document` property/method surface used to scope the DOM subset (`suspendHistory`, `saveAs`, `exportDocument`, collections). Community mirror of the scripting reference.
- Cross-reference `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`), which contains the underlying ExtendScript overview, object-model table, proposal sketch, and shared Sources.

## Open questions

- **Engine is not ratified.** QuickJS (`rquickjs`) is recommended; resolve with a benchmark/binding spike covering DOM ergonomics, Qt lifecycle, startup latency, and Linux packaging (prebuilt vs `bindgen`).
- **Required `.jsx` compatibility depth.** Which subset must run unmodified? Resolve by surveying real CS6 scripts (Adobe samples, common libraries) and measuring E4X/ScriptUI/`File` usage.
- **E4X strategy.** Shim, transpile, or reject-with-diagnostic is undecided; the shim policy in this spec is "reject with a named diagnostic" until evidence demands otherwise.
- **ScriptUI scope.** Full Qt-backed ScriptUI vs a minimal dialog API vs none. Resolve with user research and the `11-cross-cutting` accessibility policy.
- **Action-descriptor serialization and `.atn`.** Whether Kooka Pictura actions interoperate with Adobe `.atn`/droplets or only its own JSON format.
- **Four-char id table provenance.** The mapping table must be built without copying Adobe assets; confirm an independent derivation path and document it under the licensing policy.
- **Sandbox model.** In-process capability checks versus out-of-process/WASM for untrusted scripts; resolve in `11-cross-cutting/security-and-sandboxing`.
- **Timeout default.** 30 s matches the proposal in `ARCH-011` but is unvalidated against real batch scripts; measure against representative scripts and revisit.
- **Engine fallback packaging.** Shipping QuickJS *and* Rhai doubles the surface; whether the alternative engine ships at all is deferred to Phase 5.
