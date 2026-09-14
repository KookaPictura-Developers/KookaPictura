# Automation Plugins

- **Spec ID:** `AUTO-013`
- **Status:** `Draft`
- **Parity tier:** `Core` for the `File > Automate` surface and the in-process automation model; loading Adobe `.8li`/`.8ly` automation binaries is `Non-goal (Linux)`.
- **New in CS6:** `Changed` — CS6 restored **Contact Sheet II** and **PDF Presentation** as Automate options (they had been moved to scripts/Bridge) and recorded brush strokes in actions. The automation plug-in ABI itself is unchanged from CS5.
- **Depends on:** `AUTO-012` (plugin-sdk), `ARCH-011` (plugin-and-scripting-abi), `AUTO-010` (scripting engine), `09-automation/actions`, `09-automation/batch-processing`, `09-automation/droplets`.

## CS6 behavior

Automation plug-ins add commands to the **`File > Automate`** submenu. They are one of the plug-in kinds in `AUTO-012`; their distinguishing feature is the entry point (`AutoPluginMain`) and the caller/selector model rather than the filter/format `PluginMain` selector loop.

The CS6 `File > Automate` menu mixes Adobe automation plug-ins and script-backed commands. Commands documented in the CS6 Help/tables include:

| Command | Kind | Notes |
|---|---|---|
| `Batch...` | Automation | Runs an action over a folder / opened files / Bridge selection. |
| `Create Droplet...` | Automation | Builds a droplet application from an action. |
| `PDF Presentation` | Automation | Restored as an Automate option in CS6. |
| `Contact Sheet II` | Automation | Restored as an Automate option in CS6. |
| `Photomerge...` | Automation | Panorama composition; can also be invoked from Bridge. |
| `Merge To HDR Pro...` | Automation | HDR merge; also from Bridge. |
| `Conditional Mode Change...` | Automation | Inserts a mode-change condition. |
| `Crop And Straighten Photos` | Automation | Splits scanned multi-image sheets. |
| `Picture Package...` | Optional automation plug-in | Download/install separately (Windows/macOS). |
| `Pattern Maker...` | Optional plug-in | Optional download; filter-adjacent. |

Observable behaviour this spec must preserve:

- Automation commands run synchronously from the menu and may show a modal dialog before operating (except when suppressed for batch).
- They operate on the current document, opened files, a chosen folder, or a Bridge selection (Batch's `Source`: Folder / Import / Opened Files / Bridge). On Linux there is no Bridge, so the Bridge source is non-goal.
- Batch processing does not auto-save: the target action must contain a `Save As` step, or the destination "Save and Close" with "Override Action Save As Commands" must be chosen.
- For batch performance CS6 advises reducing saved history states and deselecting "Automatically Create First Snapshot".
- A third-party import plug-in that does not support multiple documents may fail during batch or action playback.
- `Create Droplet` produces a drag-and-drop target (droplet) that applies an action to dropped images/folders; the droplet can be saved to disk.
- Automation commands can be invoked from scripts (`app.batch`, `app.doAction`) and from Bridge (`Tools > Photoshop > ...`).

### PICA automation entry and event handling

Automation plug-ins use the PICA string caller/selector entry point (from the Adobe SDK and the PICA API reference, and the community sample):

```c
DLLExport SPAPI SPErr AutoPluginMain(const char* caller,
                                     const char* selector,
                                     void* message);
```

The sample dispatch:

```c
SPMessageData* basicMessage = (SPMessageData*)message;
sSPBasic = basicMessage->basic;

if (sSPBasic->IsEqual(caller, kSPInterfaceCaller)) {
    if (sSPBasic->IsEqual(selector, kSPInterfaceAboutSelector))    DoAbout(basicMessage->self, AboutID);
    if (sSPBasic->IsEqual(selector, kSPInterfaceStartupSelector))  return kSPNoError;
    if (sSPBasic->IsEqual(selector, kSPInterfaceShutdownSelector)) return UninitializePlugin();
}
if (sSPBasic->IsEqual(caller, kPSPhotoshopCaller)) {
    if (sSPBasic->IsEqual(selector, kPSDoIt)) {
        PSActionsPlugInMessage* msg = (PSActionsPlugInMessage*)message;
        /* run the automation command */
    }
}
```

Event/selector summary:

| Caller | Selector | Meaning |
|---|---|---|
| `kSPInterfaceCaller` (`"SP Interface"`) | `kSPInterfaceStartupSelector` (`"Startup"`) | Allocate globals, add menu items; happens at app launch. |
| `kSPInterfaceCaller` | `kSPInterfaceAboutSelector` (`"About"`) | Show the About box. |
| `kSPInterfaceCaller` | `kSPInterfaceShutdownSelector` (`"Shutdown"`) | Flush preferences/files; do not destroy exported suite data (may be called after shutdown by another plug-in). |
| `kPSPhotoshopCaller` | `kPSDoIt` | Execute the command the user selected. |
| `kSPAccessCaller` (`"SP Access"`) | reload / unload | Brackets plug-in life; set up / save state. Acquire no non-PICA suites here. |

Automation plug-ins reach document operations through the **ActionManager**: the SDK's `PSActionDescriptorSuite` (and `PIUXPSuite`/`PIActionDescriptor` in later SDKs) builds event descriptors; the plug-in calls into Photoshop the same way an action or script would. The CS6 automation plug-in is therefore an *action driver* packaged as a native command.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `File > Automate > <name>` | menu | — | Automation plug-ins and script-backed commands. |
| `File > Automate > Batch...` | dialog | — | Set/Action, Source (Folder/Import/Opened Files/Bridge), Destination, file naming; "Override Action Save As". |
| `File > Automate > Create Droplet...` | dialog | — | Save droplet location, action, destination, naming, "Suppress File Open Options Dialogs". |
| `File > Automate > PDF Presentation` | dialog | — | Restored in CS6. |
| `File > Automate > Contact Sheet II` | dialog | — | Restored in CS6. |
| `File > Automate > Photomerge...` | dialog | — | Source Files (Files/Folders), Layout, Blend, etc. |
| `File > Automate > Merge To HDR Pro...` | dialog | — | Also Bridge-invoked in CS6. |
| `File > Automate > Conditional Mode Change...` | dialog | — | Source/target mode conditions. |
| `File > Automate > Crop And Straighten Photos` | command | — | No dialog; splits the scan. |
| `File > Automate > Picture Package...` | dialog | — | Optional install in CS6. |
| `File > Automate > Pattern Maker...` | dialog | — | Optional install; filter-adjacent. |
| Droplet `.exe`/app | OS desktop object | — | Applies its action to dropped files/folders. |
| Automation progress | widget | — | Cancel maps to `OP_ERR_CANCELLED`. |

## Parameters & ranges

Automation commands expose per-command dialogs; the cross-cutting controls are:

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Batch `Source` | enum | Folder | Folder, Import, Opened Files, Bridge | Bridge is non-goal on Linux. |
| Batch `Destination` | enum | None | None, Save And Close, Folder | "None" leaves files open. |
| Batch file naming | template | document name + extension | tokens (document name, extension, serial, date) | Override behaviour follows the Help. |
| Batch `Include All Subfolders` | bool | false | on/off | Recursive folder walk. |
| `Suppress File Open Options Dialogs` | bool | false | on/off | Droplet/Play area. |
| `Override Action Save As Commands` | bool | false | on/off | Requires a Save As step in the action. |
| `Suppress Color Profile Warnings` | bool | false | on/off | Batch Play area. |
| Progress reporting | — | on | on/off | `doProgress`/`doProgressTask`. |
| Automation plug-in `Name` | UTF-8 string | — | 1–47 bytes | Menu label; CS6 PiPL limit. |
| Automation `Category` | string | `Kooka Pictura Automate` | free text | Grouping; CS6 shows a flat Automate menu. |

## Algorithms & pipeline

### Automation command lifecycle (proposed)

```text
host startup:
  discover automation plug-ins (same scan as AUTO-012)
  call entry(kSPInterfaceCaller, kSPInterfaceStartupSelector)   # register menu item
menu invocation:
  call entry(kPSPhotoshopCaller, kPSDoIt) with a message block
      - first call: show dialog (unless displayDialogs == NO / batch)
      - build an ActionDescriptor / command batch
      - execute through the shared command bus (one or more transactions)
      - stream progress; honour cancel
  commit or abort
host shutdown:
  call entry(kSPInterfaceCaller, kSPInterfaceShutdownSelector)
About:
  call entry(kSPInterfaceCaller, kSPInterfaceAboutSelector)
```

In the proposed Kooka Pictura ABI (`ARCH-011`) the same lifecycle is expressed as:

| CS6 | Kooka Pictura |
|---|---|
| `AutoPluginMain(caller, selector, message)` | `OpPluginApi::process(ctx, req, res)` with `req.selector` |
| `kSPInterfaceStartupSelector` | `OP_SEL_AUTOMATION_STARTUP` during `init` |
| `kPSDoIt` | `OP_SEL_AUTOMATION_RUN` |
| `kSPInterfaceAboutSelector` | `OP_SEL_ABOUT` |
| `kSPInterfaceShutdownSelector` | `OpPluginApi::shutdown` |
| `SPMessageData { self, globals, basic }` | `OpPluginCtx` + `OpHostApi` |
| `PSActionDescriptorSuite` | `crate::action::Descriptor` over `crate::command` |

Automation plug-ins get `OP_CAP_AUTOMATION` plus the capabilities of the operations they perform (`OP_CAP_DOC_MUTATE`, `PIXEL_*`, `FILESYSTEM` opt-in for Batch). A plug-in declaring automatable commands is registered under `File > Automate`.

### Batch pipeline

```text
batch(set, action, source, destination, naming, overrides):
  files = resolve_source(source)                 # folder(+subfolders) / opened / import
  begin_progress(total = len(files))
  for f in files:
      if cancelled: abort, return OP_ERR_CANCELLED
      doc = open(f)                               # suppress open-options dialogs if set
      play_action(doc, set, action)               # existing Actions engine
      if destination == None:        keep open
      if destination == SaveAndClose: save per overrides; close
      if destination == Folder:       save_as(naming(f), out_dir); close
      release(doc)                                # reduce history per CS6 advice
      progress_step()
  commit history per file; the batch as a whole is not one undo

create_droplet(action, destination, naming, suppress_open_options):
  emit a desktop launcher (Linux: .desktop + handler, or a small wrapper)
  that re-runs batch on dropped paths (a "droplet" equivalent)
```

The droplet is the CS6 desktop object; on Linux the closest equivalents are a `.desktop` file plus an `op-pictura --droplet <preset>` entry point, or a tiny shell wrapper. The spec proposes the CLI entry point and a generated `.desktop` launcher.

### Event handling and reentrancy

- Automation commands are driven by the same command bus and event stream as scripts and the UI. A command that emits events can fire notifiers, but notifier execution is deferred to the command loop (no reentrant `executeAction`), consistent with `ARCH-011`.
- `displayDialogs = NO` (batch/script context) suppresses modal automation dialogs; commands must either proceed with defaults or return `OP_ERR_CANCELLED`.
- Startup handlers must not block or show UI; CS6 loads plug-ins at launch, so registration work belongs there.

### Mapping built-in Automate commands

| CS6 command | Proposed implementation |
|---|---|
| Batch | Reuse the actions engine + a batch driver in the core (not a separate plug-in). |
| Create Droplet | Emit a `.desktop` launcher/CLI preset. |
| PDF Presentation | Core document/render + PDF export; automation UI on top. |
| Contact Sheet II | Core layout + image compositing; automation UI. |
| Photomerge | Core stitching engine (see the Photomerge feature spec). |
| Merge To HDR Pro | Core HDR merge (see 32-bit HDR specs). |
| Conditional Mode Change | Action step + condition evaluator. |
| Crop And Straighten Photos | Core scan-splitting algorithm. |
| Picture Package / Pattern Maker | Optional; separate feature specs. |

Kooka Pictura does **not** bundle Adobe's automation binaries; each built-in is reimplemented as a core feature or a script, and the `File > Automate` menu is populated from the same registry as other plug-ins.

## Rust module mapping

- `crate::automation::registry` — discovered automation commands; menu entries; `AutomationCommand { id, name, category, source: Builtin | Plugin | Script }`.
- `crate::automation::dispatch` — maps `OP_SEL_AUTOMATION_*` selectors to handlers; startup/shutdown/about lifecycle.
- `crate::automation::batch` — `BatchRequest { set, action, source, destination, naming, overrides }`, folder walker, progress/cancel, per-file open/close.
- `crate::automation::droplet` — preset serialization + `.desktop`/CLI launcher generation.
- `crate::automation::builtins` — Batch, Data Sets export, PDF Presentation, Contact Sheet, Photomerge, HDR, Conditional Mode Change, Crop And Straighten.
- Reuses `crate::action` (Actions engine), `crate::command`, `crate::plugin` (for C-ABI automation plug-ins), `crate::script` (`app.batch`, `app.doAction`).
- `crate::formats` for output encoding; `crate::render` for PDF/contact-sheet compositing.

## Qt6 component mapping

- `AutomateMenu` — built into `QMenuBar` from `AutomationRegistryModel`.
- `BatchDialog` — Set/Action combos, Source radio/`QComboBox` (+ folder chooser), Destination combo, `QLineEdit` naming, override checkboxes, progress area with `QProgressBar` and Cancel.
- `CreateDropletDialog` — droplet path, action selection, destination/naming, suppress-open-options.
- `PdfPresentationDialog`, `ContactSheetIIDialog`, `PhotomergeDialog`, `MergeToHdrProDialog`, `ConditionalModeChangeDialog` — per-command dialogs; widgets over QML because they are dense, modal, CS6-styled.
- `AutomationProgressDialog` — `QProgressDialog` bridged to `doProgress`/cancel → `OP_ERR_CANCELLED`.
- `DropletLauncher` — a generated `.desktop` file (plus a small `op-pictura --droplet <preset>` handler) presented to the user.

## Data-model impact

- **New runtime nodes:** `AutomationCommand` (registry), `BatchRequest`/`DropletPreset` (transient or saved to preferences, not PSD).
- **Preferences:** saved batch/droplet presets (an Kooka Pictura extension; CS6 stored droplets as files/objects, not preferences).
- **Undo:** each processed document mutates through the normal command history; the batch as a whole is **not** one undo. Individual automation commands (e.g. Crop And Straighten) are one transaction on the active document.
- **Serialization:** no PSD impact; droplet presets serialise to the project's preset format. Generated `.desktop` files carry no document metadata.
- **Events:** automation commands emit the same command events other systems see, so notifiers/Script Events Manager can observe them; no new event type is required beyond per-command ids.

## Edge cases

- **No Bridge on Linux:** Batch's `Bridge` source and Bridge-invoked Photomerge/HDR are non-goal; the UI must omit or disable them rather than fail.
- **Action without a Save As step:** Batch with `Override Action Save As Commands` / `Save and Close` must error with the CS6 guidance (the action must contain a Save As step).
- **Third-party import plug-ins and multi-document batch:** a plug-in that cannot acquire multiple documents must fail gracefully for that file, not abort the whole batch without reporting.
- **Cancel during batch:** abort current file cleanly, keep already-saved outputs, report how many completed.
- **Open with options dialogs:** `Suppress File Open Options Dialogs` must be honoured (CS6 recommends it for droplets).
- **History/memory during batch:** release documents and prune history between files; honor the CS6 advice to reduce saved history states.
- **`displayDialogs = NO`:** no modal UI; commands either proceed with defaults or cancel.
- **Startup handler errors:** a failing automation plug-in must not prevent other plug-ins loading (`AUTO-012` isolation rules).
- **Droplet naming collisions:** apply the same per-file naming/overwrite rules as Batch.
- **PSB/huge files and many-file batches:** stream, bound memory, and surface progress; do not load all files at once.
- **Unicode paths and file naming tokens:** preserve case/extension correctly on Linux.

## Parity acceptance criteria

1. Given an automation plug-in registered at startup, its entry appears under `File > Automate` and invoking it dispatches `OP_SEL_AUTOMATION_RUN`.
2. Given Batch with a folder source and an action containing a Save As step, every matching file is processed and saved to the destination with the chosen naming.
3. Given Batch with `Include All Subfolders`, nested folders are processed.
4. Given Batch with no Save As step and "Override Action Save As Commands", the command reports the CS6 "action must contain a Save As step" error and does not silently drop files.
5. Given `Create Droplet`, a launchable object is produced that, when a file is dropped on it, applies the same action as Batch and writes output to the chosen destination.
6. Given Cancel mid-batch, processing stops within one file, completed outputs remain, and the count of completed files is reported.
7. Given `displayDialogs = NO`, a Batch run completes without any modal dialog.
8. Given a plug-in dropped as `.8li`/`.8ly`, it is not loaded and the manager reports the Adobe-binary non-goal (shared with `AUTO-012`).
9. Given `Photomerge...` and `Merge To HDR Pro...`, the Automate entries exist and open the corresponding (new) core dialogs; Bridge-only invocation is absent.
10. Given the same batch run twice, outputs are byte-identical for deterministic actions.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — Automate/Actions/Batch/Droplet chapters and What's New: `File > Automate > Conditional Mode Change`, `Merge To HDR Pro`, `Photomerge`, `Crop And Straighten Photos`, `Batch`; `Create Droplet` and droplet guidance (Save As step, Suppress File Open Options Dialogs); Contact Sheet II and PDF Presentation restored; Picture Package optional plug-in; batch-source/Bridge notes. Fetched via `curl` + `pdftotext`.
- `https://raw.githubusercontent.com/sonictk/ps_cpp_recipes/master/recipes/1_hello_world/src/tutorial_automation_main.cpp` — `AutoPluginMain` dispatch: `kSPInterfaceCaller` + Startup/About/Shutdown, `kPSPhotoshopCaller` + `kPSDoIt`, `PSActionsPlugInMessage`.
- `https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/cpp-pluginsdk` — Automation entry point named `AutoPluginMain`, `SPMessageData`, `SPBasicSuite`, ActionDescriptor suites used by automation/C++ plug-ins.
- `https://adobesdk.neocities.org/pluginsdk/documentation/PICA.pdf` — PICA caller/selector semantics, `SPMessageData`, startup/shutdown/about selectors, access reload/unload bracketing, suite acquisition.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` (Scripting chapter) — `app.batch`, `app.doAction`, Image Processor, and the Scripts-menu built-ins that overlap with Automate.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-Scripting-Guide.pdf` — ActionManager/`executeAction` mechanics that automation plug-ins use to drive document operations.
- Normative ABI: `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`).

## Open questions

- **Automation plug-in PiPL specifics.** Whether automation plug-ins use `SPEA`/`8LIZ` and which PiPL properties register the Automate menu entry needs the CS6 SDK headers; resolve with the SDK or API guide.
- **Droplet representation on Linux.** `.desktop` + CLI handler vs a portable wrapper vs an internal preset list. Resolve with the build/packaging spec (`01-architecture/build-and-packaging`).
- **Which built-ins are core vs plug-in.** Whether Batch/PDF Presentation/Contact Sheet are core features or shipped as optional automation plug-ins is a packaging decision.
- **Bridge-sourced Batch replacement.** What (if anything) replaces "Source: Bridge" once a file browser exists (`AUTO-014`); likely the file browser's selection.
- **`app.batch` DOM surface.** Exact parameter/option names to expose for script parity with CS6 (`BatchOptions` in the scripting reference).
- **Droplet file-open options suppression default.** Whether Kooka Pictura defaults it on for generated droplets.
- **Automation progress for scripts.** Whether script-invoked batch shows progress when `displayDialogs = NO`.
