# Plugin SDK

- **Spec ID:** `AUTO-012`
- **Status:** `Draft`
- **Parity tier:** `Core` for the in-process plug-in model and its user-visible surface; loading Adobe binary plug-ins is `Non-goal (Linux)`.
- **New in CS6:** `No` — the native C/C++ plug-in ABI is unchanged from CS5. CS6 adds optional plug-ins distributed separately (e.g. Pattern Maker, Picture Package, Extract) and keeps the Camera Raw 7 plug-in; none of that changes the ABI. UXP did not exist in CS6.
- **Depends on:** `ARCH-011` (plugin-and-scripting-abi, which is the normative ABI design), `01-architecture/rust-core-design`, `01-architecture/document-model`, `01-architecture/undo-history`, `11-cross-cutting/security-and-sandboxing`, `00-overview/licensing-and-independent-creation`.

## CS6 behavior

Photoshop CS6 is extended in-process by the closed **Plug-in SDK**: native shared libraries that Photoshop discovers in its `Plug-ins` folder (recursively, including shortcuts/aliases to libraries elsewhere) and in an optional **Additional Plug-ins Folder** chosen under `Edit > Preferences > Plug-ins`. Adobe's own import/export/effects plug-ins ship in sub-folders of the Plug-ins folder; third-party plug-ins follow their own installer.

User-visible consequences:

- Installed plug-ins appear as options in the `Import`/`Export` menus, as file formats in the `Open`/`Save As` dialogs, and as filters in `Filter` sub-menus.
- If the plug-in list grows too long, Photoshop may not be able to show all plug-ins in their proper menus; newly installed plug-ins then appear under `Filter > Other`.
- Renaming a plug-in file, folder, or directory with a leading tilde `~` suppresses its loading.
- `Help > About Plug-in` (Windows) / `Photoshop > About Plug-in` (macOS) shows per-plug-in info.
- Some older optional 64-bit macOS plug-ins require Photoshop to run in 32-bit mode.
- Third-party filters that support Smart Filters can be applied non-destructively to a smart object; Extract and Pattern Maker are explicitly excluded from Smart Filter support.

`ARCH-011` documents the plug-in types, entry points, the PiPL, the suite model, and why Adobe binaries cannot load on Linux; this spec is the plug-in-SDK-facing companion: it fixes the type taxonomy, the `PluginMain` selector model, PiPL resources, `SPBasicSuite` host services, the Linux incompatibility argument, and the proposed Rust C-ABI model (defined in `ARCH-011`).

### Plug-in types

| Type | Windows ext. | Mac type code | Entry point |
|---|---|---|---|
| Filter | `.8bf` | `8BFM` | `PluginMain(int16 selector, FilterRecordPtr, intptr_t*, int16*)` |
| Import / acquire | `.8ba` | `8BAM` | `PluginMain` with `ImportRecordPtr` |
| Export | `.8be` | `8BEM` | `PluginMain` with `ExportRecordPtr` |
| File format | `.8bi` | `8BIF` | `PluginMain(int16 selector, FormatRecordPtr, intptr_t*, int16*)` |
| Automation | `.8li`, `.8ly` | `8LIZ` | `SPErr AutoPluginMain(const char* caller, const char* selector, void* message)` |
| Color picker | `.8bc` | `8BCM` | picker entry point |
| Extension | `.8bx` | `8BXM` | extension entry point |
| General | `.8bp` | `8BPI` | general entry point |
| Parser / selection | `.8by` / `.8bs` | `8BYM` / `8BSM` | Adobe-only |

The taxonomy comes from Wikipedia's plug-in table (community source) and the SDK documentation; the entry-point signatures are corroborated by the Adobe C++ SDK page and community samples.

### The `PluginMain` selector model

The host communicates through **one entry point called repeatedly with a selector** (a 16-bit opcode for filter/format plug-ins, or caller/selector strings for PICA-based automation plug-ins).

Filter/format entry point (from WebPShop's dispatch, a public reference implementation):

```c
DLLExport MACPASCAL void PluginMain(const int16 selector,
                                    FormatRecordPtr formatParamBlock,
                                    intptr_t* dataPointer,
                                    int16* result);
```

- `selector` identifies the stage: `formatSelectorAbout`, then per operation (`Read`, `Options`, `Estimate`, `Write`, `ReadLayer`, `WriteLayer`, `FilterFile`) a `Prepare → Start → Continue → Finish` sequence.
- `formatParamBlock` is the current `FormatRecord` (or an `AboutRecord` for the about selector).
- `dataPointer` is a persistent handle Photoshop passes across the successive calls of one logical operation; the plug-in allocates its `Data` struct on first call and Photoshop frees it eventually.
- `result` is an error code; `noErr` (0) means success.
- The whole body is wrapped in `try/catch`; an uncaught exception sets `result = -1`, so no C++ exception reaches the host.

Filter plug-ins instead receive a `FilterRecord` carrying image geometry, plane counts, input/output buffers, row bytes, a selection mask, callbacks, and the host's `SPBasicSuite`. A `Data` block is threaded through the same way via the `intptr_t*` argument.

### PICA caller/selector model (automation and legacy plug-ins)

Adobe's cross-application manager is **PICA** (Plug-in Component Architecture), described in the PICA API reference. Automation plug-ins and PICA-era plug-ins use string callers and selectors:

```c
DLLExport SPAPI SPErr AutoPluginMain(const char* caller,
                                     const char* selector,
                                     void* message);
```

- `caller` names the sender/category: `kSPAccessCaller` (`"SP Access"`), `kSPInterfaceCaller` (`"SP Interface"`), a host application caller (`kPSPhotoshopCaller` in Photoshop), or another plug-in's identifier.
- PICA selectors: `kSPAccessReloadSelector`, `kSPAccessUnloadSelector`, `kSPInterfaceStartupSelector`, `kSPInterfaceShutdownSelector`, `kSPInterfaceAboutSelector`.
- The Photoshop caller selects the actual operation; `kPSDoIt` runs the command's action. The message is an `SPMessageData` (or a PS-specific extension such as `PSActionsPlugInMessage`) whose first member is the `SPMessageData`:

```c
typedef struct {
    SPPluginRef self;      /* the running plug-in */
    void* globals;         /* 4-byte plug-in private value, preserved across unload */
    SPBasicSuite* basic;   /* acquire/release any other suite */
} SPMessageData;
```

- The `self` reference is used to register suites/adapters and to be recalled; `globals` is a 4-byte slot (typically a pointer to plug-in state); `basic` yields every other suite.
- Loading brackets every plug-in: an access/reload message is first, access/unload is last; a plug-in must not acquire or release suites other than PICA built-ins during access.

### PiPL resources

A plug-in is identified before any code runs by its **PiPL** (Plug-in Property List) resource. PICA recognises only files with a valid PiPL; file type/extension is irrelevant to discovery. A PICA plug-in's PiPL must contain at least:

- **`kind`** — the plug-in type. PICA itself looks for kind `SPEA`; Photoshop defines its own kinds per type.
- **`ivrs`** (interface version) — calling conventions; PICA's reference says it is currently `2`.
- **Code descriptor** — where the code lives, per platform (68k/PowerPC/Windows in the PICA guide; later Photoshop uses a per-platform entry symbol). Multiple descriptors make a "fat" plug-in.
- **`expt`** (export property) — optional; names and API versions of suites the plug-in provides, so PICA can order loading correctly.

PiPLs are compiled per platform: Apple `Rez` (byte order is always Mac big-endian) or Windows `pipltool.exe`/`cnvtpipl.exe` feeding a `.rc` resource. The property set for Photoshop plug-ins includes Kind, Name, Category, Entry Point, version, and host-requirement flags.

### Host services: `SPBasicSuite` and suites

Host functionality is exposed as **suites**: named, versioned vtables of function pointers. A plug-in holds only the `SP Basic Suite` at entry and acquires everything else by name + version:

```c
SPBasicSuite *sBasic = message->d.basic;
MyAppMenuSuite *sMyAppMenu;
error = sBasic->AcquireSuite(kMyAppMenuSuite, kMyAppMenuVersion, &sMyAppMenu);
/* use sMyAppMenu->... */
sBasic->ReleaseSuite(kMyAppMenuSuite, kMyAppMenuVersion);
```

The PICA reference's `SPBasicSuite` surface is `AcquireSuite`, `ReleaseSuite`, `IsEqual`, `Undefined`; every `SPErr` is a four-byte error code. Photoshop-specific suites seen in the SDK/community surface include `PSActionDescriptorSuite`, `PSChannelPortsSuite`, and the CC-era `PIUXPSuite` (`SendUXPMessage`, `AddUXPMessageListener`, `RemoveUXPMessageListener`). Suites are independently versioned, so new versions do not break old plug-ins. The API is documented as **not thread-safe**; plug-ins run on the host thread. Plug-ins must be built with the compiler/toolchain Photoshop was built with (MSVC version is stated per release in the SDK).

### Why Adobe binaries cannot load on Linux

Reusing the ABI is impossible without reimplementing the entire closed host:

1. **Binary format and resources.** Plug-ins are PE (Windows) or Mach-O/Carbon code fragments (macOS) with platform resource sections. Linux has no `.8bf`/`.8bi` consumer and no PE/Mach-O resource loader equivalent.
2. **Toolchain-specific ABI.** The calling conventions (`MACPASCAL`/`SPAPI`/`WINAPI`) and C++ name mangling are MSVC/Apple-specific; ELF/GCC/Clang does not match.
3. **Resource pipeline.** PiPL compilation uses `Rez`/`pipltool`/`cnvtpipl` + `.rc`, all macOS/Windows-only. Carbon-era types (`Str255`, `Rect`, `Fixed`, `OSType`) and always-big-endian PiPL archives are not ABI-compatible with Linux.
4. **Closed host interface.** `SPBasicSuite` and its suites are Adobe-proprietary with no published open specification; the SDK is license-gated and restricts redistribution and analysis.
5. **Not thread-safe, host-thread bound.** The contract assumes Photoshop's single host thread and its lifetime/unload rules.

Kooka Pictura therefore reads/writes Adobe *file formats* but does not execute Adobe *binary* plug-ins. A stock `.8bf`/`.8bi` dropped in a directory is skipped with a reported "unsupported format (Adobe binary compatibility is a non-goal)".

### What Kooka Pictura implements

The proposed Rust-native C ABI is specified in `ARCH-011` (single versioned entry symbol `kookapictura_plugin_entry_v1`, `#[repr(C)]` structs, version/size header, capability negotiation, opaque `OpDocId`/`OpLayerId` handles, error codes, allocator contract, no unload by default, in-tree `trait Plugin` + `CAbiAdapter`). This spec does not duplicate it; it fixes the plug-in-kind → capability mapping and the migration of the CS6 user-visible behaviours.

| CS6 plug-in kind | Proposed capability bit(s) | Menu/UI |
|---|---|---|
| Filter | `OP_CAP_FILTER` (+ `PIXEL_READ/WRITE`) | `Filter > <category>` |
| Import/acquire | `OP_CAP_IMPORT` | `File > Import >` |
| Export | `OP_CAP_EXPORT` | `File > Export >` |
| File format | `OP_CAP_FORMAT_READ` / `_WRITE` | `Open`/`Save As` type lists |
| Automation | `OP_CAP_AUTOMATION` | `File > Automate >` (see `AUTO-013`) |
| Extension | `OP_CAP_DOC_MUTATE` (UI contribution) | `Window > Extensions` (see `AUTO-014`) |
| General | capability-negotiated | varies |
| Color picker / parser / selection | deferred | — |

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Filter > <category> > <name>` | menu | — | Filter plug-ins grouped by PiPL `Category`; overflow to `Filter > Other`. |
| `File > Import > <name>` | menu | — | Import/acquire plug-ins. |
| `File > Export > <name>` | menu | — | Export plug-ins. |
| `Open` / `Save As` type list | dialog option | — | File-format plug-ins. |
| `File > Automate > <name>` | menu | — | Automation plug-ins (`AUTO-013`). |
| `Edit > Preferences > Plug-ins` | preference page | — | Additional Plug-ins Folder (CS6 parity). |
| `Help > About Plug-in > <name>` | menu | — | About boxes. |
| Plug-in manager (new) | dialog | — | List/enable/disable Kooka Pictura native plug-ins, capabilities, load errors. |
| `~`-prefix suppression | filesystem convention | — | Leading `~` on file/folder name skips loading (parity). |
| Load-error banner (new) | widget | — | Non-modal warning for ABI/capability/format failures. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Plug-in `kind` | enum | — | filter, import, export, format, automation, extension, general, colorPicker, parser, selection | Derived from capabilities in Kooka Pictura. |
| Plug-in `Name` | UTF-8 string | — | 1–47 bytes | CS6 PiPL name limit enforced for menu parity. |
| `Category` | UTF-8 string | `Kooka Pictura Plugins` | free text | Filter menu grouping. |
| ABI major/minor | u32 / u32 | `1` / `0` | host-enforced | Major mismatch refuses load. |
| `struct_size` | usize | — | ≥ host minimum | Forward-compatible trailing fields. |
| Capability bitset | u64 | plug-in-declared | see `ARCH-011` | Host intersects with granted set. |
| `Additional Plug-ins Folder` | path | empty | one folder | CS6 allows one extra folder. |
| Suppress-load prefix | char | `~` | file/folder name | Parity behaviour. |
| ROI / pixel format | per call | — | 8/16/32-bit, Gray/RGB/CMYK/Lab, explicit row bytes | From `ARCH-011` ABI. |

## Algorithms & pipeline

### Host load sequence (CS6 analogue)

```text
1. scan <app>/Plug-ins recursively; follow shortcuts/aliases
2. scan the Additional Plug-ins Folder (Preferences > Plug-ins)
3. skip any entry whose name begins with '~'
4. identify each candidate by its metadata (PiPL in CS6; a manifest in Kooka Pictura)
5. call the single entry symbol; validate ABI header; negotiate capabilities
6. register by kind; populate Filter/Import/Export/Automate/Extensions menus
7. dispatch later via one entry point + selector (same call repeatedly)
8. shutdown: call shutdown; do NOT unload (see ARCH-011 no-dlclose rule)
```

The metadata channel replaces the PiPL: Kooka Pictura uses the versioned `OpPluginApi` struct returned by `kookapictura_plugin_entry_v1` (id, name ≤ 47 bytes, category, version, wanted/granted caps) rather than a platform resource. No `Rez`/`pipltool` step exists; this is the key portability win.

### `PluginMain`-style dispatch in the proposed ABI

CS6's selector loop maps onto `OpPluginApi::process`:

| CS6 | Kooka Pictura |
|---|---|
| `PluginMain(selector, record, data, result)` | `process(ctx, req: *const OpProcessRequest, res: *mut OpProcessResult)` |
| `int16 selector` | `req.selector` (u32 per kind; e.g. read/write Prepare→Start→Continue→Finish) |
| `FormatRecord*` / `FilterRecord*` | `req.doc`, `req.layer`, `req.roi`, `req.params` |
| `intptr_t* data` (persistent `Data`) | `OpPluginCtx` (`ctx`), opaque to host |
| `int16* result` | `OpProcessResult.flags` + `OpStatus` |
| `AboutRecord` special case | a dedicated `OP_SEL_ABOUT` selector |

Filter-plug-in pixel access is the `doc_get_pixels`/`doc_commit_pixels` pair in `OpHostApi` (capability-gated), bounded by `req.roi`; format-plug-in read/write uses `OP_CAP_FORMAT_READ/WRITE` and a `FormatRecord`-style geometry struct mapped to the internal image format.

### PiPL → metadata migration

```text
CS6:      PiPL resource (kind, ivrs, code descriptor, expt, name, category)
O-P:      OpPluginApi { id, name, category, version, wanted_caps, granted_caps }
          + ABI header { abi_major, abi_minor, struct_size }
```

The 47-byte name limit and category grouping are preserved for menu parity; everything else is replaced by the struct.

### Why a new ABI instead of an emulator

An emulation layer would have to fake PE/Mach-O loading, Carbon resource sections, MSVC mangling, PiPL parsing, the full `SPBasicSuite`/suite catalogue, and host-thread semantics, then keep the fidelity high enough for arbitrary third-party plug-ins. That is strictly more work than the new, documented ABI and is legally fraught (`licensing-and-independent-creation`). Non-goal.

## Rust module mapping

The ABI layer lives in `ARCH-011`; this spec adds the SDK-kind glue:

- `crate::plugin::kind` — `PluginKind { Filter, Import, Export, Format, Automation, Extension, General, ... }`, capability derivation, menu routing.
- `crate::plugin::metadata` — `PluginMetadata { id, name, category, version }`, 47-byte name truncation/validation.
- `crate::plugin::discovery` — directory scan, `~` suppression, additional-folder preference, duplicate-id policy.
- `crate::plugin::dispatch` — `SelectorRoute` per kind; `OP_SEL_ABOUT` handling; Prepare/Start/Continue/Finish state machine mapping for format/filter kinds.
- `crate::plugin::manager` — enable/disable state, load errors, About-box data.
- Reused from `ARCH-011`: `crate::plugin::{abi, loader, registry, host_api, Plugin, CAbiAdapter, sandbox}`.

## Qt6 component mapping

- `PluginManagerDialog` — table of plugins (id, kind, name, category, version, granted caps, source path, enabled); `PluginListModel` (see `ARCH-011`).
- `PluginsPreferencesPage` — the `Edit > Preferences > Plug-ins` page: Additional Plug-ins Folder chooser, reload button, note about `~` suppression.
- `AboutPluginDialog` — name/version/category/capabilities.
- `PluginErrorBanner` — non-modal warning for ABI/capability/unsupported-format failures.
- Menu models: `FilterMenuModel`, `ImportMenuModel`, `ExportMenuModel`, `FormatFilterModel` (for Open/Save dialogs) fed by the registry, with the `Filter > Other` overflow rule.

Widgets, not QML: these are docked/utility surfaces over `QMainWindow`.

## Data-model impact

- **New runtime node:** `PlugInRegistration { id, kind, capabilities, source_path, enabled }` (not persisted in PSD).
- **Preferences:** `AdditionalPluginsFolder: Option<PathBuf>`, `DisabledPluginIds: Set<String>` (Kooka Pictura's enable/disable replaces CS6's `~` convention while still honouring `~`).
- **No PSD impact:** plug-in metadata is not written into PSD/PSB; file-format plug-ins participate only through the read/write pipeline.
- **Undo:** a plug-in operation is one transaction bracketed by `undo_begin`/`undo_commit` (`ARCH-011`); `HistoryRecord::PluginOp { plugin_id, selector, roi, layer, before_hash, after_hash }`.
- **Menu state:** registry changes (enable/disable/load) update menu models; no document change.

## Edge cases

- **Non-pictura binary in the folder:** skipped silently at debug level; no crash; manager lists it as unsupported if surfaced.
- **ABI/version mismatch:** major refusal with a diagnostic; minor loads and hides unknown fields.
- **Panic across FFI:** caught at the boundary → `OP_ERR_PANIC`; plug-in disabled; host survives (`ARCH-011`).
- **Allocator mismatch:** host-allocated/host-freed and plug-in-allocated/plug-in-freed via host callbacks; never `free()` a Rust `Box` across the boundary.
- **Unload/hot-reload:** do not `dlclose`; load a new version and drain the old on idle.
- **`~` suppression:** applies to files and whole folders; must not be followed for shortcuts/aliases to outside paths.
- **Duplicate plug-in ids:** first wins; later ones reported and skipped.
- **Menu overflow:** if the filter list is too long, overflow goes to `Filter > Other` (CS6 parity).
- **Non-thread-safe plug-ins:** serialize calls on the document thread; no concurrent `process`.
- **Name length/locale:** enforce ≤ 47 bytes for parity; localized names are Kooka Pictura extensions (no PiPL localization equivalent).
- **File-format plug-in and huge PSB:** ROI/tile-budget limits apply (`ARCH-011`); no whole-canvas request above the budget.
- **Empty/1-px ROI:** `w == 0 || h == 0` rejected with `OP_ERR_INVALID_ARG`.

## Parity acceptance criteria

1. Given a filter plug-in loaded from the plug-in directory, its entry appears under `Filter > <category>` (or `Filter > Other` on overflow) and dispatches the correct selector sequence.
2. Given a file-format plug-in, its type appears in the `Open` and `Save As` dialogs and a round-trip write/read matches within the format's tolerance.
3. Given an import plug-in, it appears under `File > Import`.
4. Given a directory entry whose name begins with `~`, it is not loaded.
5. Given an additional plug-in folder set in preferences, its plug-ins load on the next launch.
6. Given a plug-in requesting a capability the host does not grant, negotiation denies it and a denied call returns `OP_ERR_CAPABILITY`.
7. Given a stock Adobe `.8bf`/`.8bi` binary in the plug-in directory, it is not loaded, no crash occurs, and the manager reports "unsupported format (Adobe binary compatibility is a non-goal)".
8. Given a plug-in `Name` longer than 47 bytes, the menu shows a truncated name and the manager warns.
9. Given a plug-in that panics in `process`, the host reports `OP_ERR_PANIC`, disables it, and the document remains usable.
10. Given `Help > About Plug-in`, the selected plug-in shows name/version/category/capabilities.

## Sources

- `https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/cpp-pluginsdk` — Automation entry `AutoPluginMain(const char* caller, const char* selector, void* message)`; filter entry `PluginMain(const int16 selector, FilterRecordPtr, intptr_t*, int16*)`; `sSPBasic = filterRecord->sSPBasic`; `AcquireSuite(kPSUXPSuite,...)`, `kPSActionDescriptorSuite`; `PiPL` component name/`PIComponentProperty` id; suites as the host API.
- `https://adobesdk.neocities.org/pluginsdk/documentation/PICA.pdf` — *Adobe Plug-in Component Architecture (PICA) API Reference*: PiPL required properties (`kind`, `ivrs`, code descriptor, `expt`), entry point and caller/selector model, `kSPAccessCaller`/`kSPInterfaceCaller` and their selectors, `SPMessageData { self, globals, basic }`, `SPBasicSuite { AcquireSuite, ReleaseSuite, IsEqual, Undefined }`, suite versioning, per-platform PiPL compilation, plug-in loading order. Adobe SDK companion document (hosted on a third-party mirror).
- `https://deepwiki.com/webmproject/WebPShop/2.1-entry-point-and-selector-dispatch` — `PluginMain` signature, `Data` persistence via `intptr_t*`, `formatSelectorAbout`/`AboutRecord`, `TryAcquireSuite`/`PSChannelPortsSuite1` v3, the complete `formatSelector*` Prepare→Start→Continue→Finish dispatch table, `try/catch` → `result = -1`.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — "Plug-ins" (pp. 54–55): Plug-ins folder, Additional Plug-ins Folder, menu placement and `Filter > Other` overflow, `~` suppression, `About Plug-in`, 32-bit-only legacy plug-ins. Fetched via `curl` + `pdftotext`.
- `https://en.wikipedia.org/wiki/Photoshop_plugin` — plug-in type table (extensions, Mac type codes) and SDK access history (community source).
- `https://github.com/sonictk/ps_cpp_recipes/blob/master/docs/index.html` — community tutorial: PiPL `.r` resource, `pipltool`/`cnvtpipl`, entry-point signatures, `sSPBasic`, PICA/Photoshop record types (referenced by `ARCH-011`; this doc used the companion automation sample below).
- `https://raw.githubusercontent.com/sonictk/ps_cpp_recipes/master/recipes/1_hello_world/src/tutorial_automation_main.cpp` — `AutoPluginMain` dispatch sample: `sSPBasic = basicMessage->basic`, `IsEqual(caller, kSPInterfaceCaller)`, startup/about/shutdown selectors, `kPSPhotoshopCaller` + `kPSDoIt`, `PSActionsPlugInMessage`.
- `https://docs.rs/libloading/latest/libloading/` and `https://doc.rust-lang.org/nomicon/ffi.html` — dynamic-loading and FFI rules cited by `ARCH-011` and reused for the proposed ABI.
- Normative design: `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`), including full Sources for the ExtendScript/ABI research.

## Open questions

- **Exact PiPL property set per Photoshop plug-in type.** The anchor points (`kind`, `ivrs`, code descriptor, `expt`, Name/Category/Entry Point) are documented, but the full Photoshop-specific property catalogue needs the CS6 SDK headers or the API-guide PDF. Resolution: obtain the SDK or the API guide; otherwise keep the Kooka Pictura metadata model independent.
- **`PIProperty` type.** The old API guide describes PiPL property entries; the exact struct named `PIProperty` was not retrieved (`ARCH-011` open question).
- **Color picker / parser / selection plug-ins.** Whether Kooka Pictura supports these kinds at all, or lists them as non-goal, is deferred.
- **Additional plug-in folder count.** CS6 allows one additional folder; whether Kooka Pictura allows more is an extension decision.
- **Localization of plug-in names.** No PiPL localization equivalent is specified; how localized names/categories reach the menu is open.
- **Value of an Adobe-plugin emulator.** Re-confirmed as non-goal; revisit only if a concrete user demand and a legal path appear.
- **Smart Filter support for plug-ins.** Which filter plug-ins are allowed on smart objects (CS6 excludes Extract/Pattern Maker) needs a per-plug-in declaration in the new metadata.
