# Plugin and Scripting ABI

- **Spec ID:** `ARCH-011`
- **Status:** `Draft`
- **Parity tier:** `Core` (scripting host and the in-process plugin model); Adobe binary plug-in loading is `Non-goal (Linux)` and is documented as such.
- **New in CS6:** `Changed` — the native plug-in ABI is unchanged from CS5; CS6 extends the ExtendScript DOM (artboards, 3D/Camera Raw surface) while the interpreter stays at ECMA-262 3rd edition + E4X. UXP did not exist in CS6.
- **Depends on:** `system-architecture`, `rust-core-design`, `document-model`, `undo-history`, `09-automation/plugin-sdk`, `09-automation/extendscript-api-surface`, `09-automation/rust-scripting-replacement`, `11-cross-cutting/security-and-sandboxing`.

## CS6 behavior

### Native plug-in architecture (Adobe SDK)

Photoshop CS6 extends through the closed C/C++ **Plug-in SDK**. Plug-ins are native dynamically loaded libraries that Photoshop discovers in its `Plug-ins` folder (recursively, following shortcuts). The host identifies a plug-in by a resource, the **PiPL** (Plug-In Property List), before any code runs. The PiPL declares the plug-in `Kind` (filter, import, export, file format, automation, color picker, extension, parser, selection, general), a display `Name` (up to 47 characters), `Category`, and the **entry-point symbol** to call for each platform.

Plug-in types differ by entry point and by file extension (Wikipedia/API-guide, community-sourced):

| Plug-in type | Windows extension | Mac type code | Entry point (documented signatures) |
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

The communication model is a **single entry point called repeatedly with a selector** (a 16-bit or string opcode). WebPShop's documented dispatch shows the file-format sequence `Prepare → Start → Continue → Finish` per operation (read, options, estimate, write, read/write layer, filter file). Filter plug-ins use a `FilterRecord` containing image geometry, plane counts, input/output buffers, row bytes, selection mask, callbacks, and a pointer to the host's `SPBasicSuite`. Format plug-ins use a `FormatRecord`; automation plug-ins receive an `SPMessageData` and dispatch on `caller`/`selector` (`kSPInterfaceCaller`, `kSPInterfaceAboutSelector`, `kSPInterfaceStartupSelector`, `kSPInterfaceShutdownSelector`, `kPSPhotoshopCaller`, `kPSDoIt`).

Host functionality is exposed to plug-ins as **suites**: named, versioned vtables acquired at runtime with `SPBasicSuite::AcquireSuite(id, version, &out)` and released with `PIUSuitesRelease()`. Examples seen in the SDK surface: `PSActionDescriptorSuite`, `PSChannelPortsSuite`, `PIUXPSuite` (CC-era). A plug-in must match the compiler/toolchain Photoshop was built with; the SDK documentation states the required MSVC version per release.

The ABI is **C++**, platform-binary-specific, and Carbon-era:

- Exported with `DLLExport` (Windows PE) or a Mac code-fragment/`CFBundle` resource; calling conventions `MACPASCAL`/`SPAPI`/`WINAPI`.
- The PiPL is compiled differently per platform: Apple `Rez` (byte order is always Mac big-endian) or Windows `pipltool.exe`/`cnvtpipl.exe` feeding a `.rc` resource.
- Data structures carry legacy Mac types (`Str255`, `Rect`, `Fixed`, `OSType`) and 16-bit `int16` selectors even in 64-bit SDKs.
- The API is documented as **not thread-safe**; plug-ins are expected to run on the host thread.
- The SDK is license-gated; redistribution and public analysis are restricted (`00-overview/licensing-and-independent-creation`).

**Why the Adobe ABI cannot be reused on Linux:** the binary format is PE/Mach-O with platform resource sections (no Linux `.8bf` consumer), the calling convention and C++ name mangling are toolchain-specific, the resource compiler (`pipltool`, `Rez`, `.rc`) is macOS/Windows-only, PiPL/`Str255`/Carbon types are not ABI-compatible with ELF/GCC/Clang, and the `SPBasicSuite` host interface plus a `PIProperty`-style metadata channel are Adobe-proprietary with no published open specification. There is no faithful way to load a stock CS6 plug-in on Linux without reimplementing the entire closed host ABI; Kooka Pictura therefore specifies a **new** ABI and treats Adobe binary compatibility as a non-goal.

> Note on `PIProperty`: Adobe's older API guide describes plug-in metadata as a property list carried in the PiPL resource. The term is used here for "plug-in property entry" (Kind, Name, Category, Entry Point, version, host requirements). The exact SDK type/struct named `PIProperty` was not retrieved during research — see **Open questions**.

### Scripting host in CS6 (ExtendScript)

Photoshop CS6 ships **ExtendScript**, Adobe's extended JavaScript interpreter. It implements ECMA-262 3rd edition (**ES3**) plus EX4 (E4X) and Adobe extensions (File/Folder, Socket, XML, operator overloading, reflection, `#include`/`#includepath`/`#target`/`#targetengine`/`#strict` preprocessor directives). It is explicitly a dead end at ES3: no native JSON, no `Array.prototype.indexOf`, no promises, no ES5/ES6 syntax.

Scripts are `.jsx` files. `#target photoshop` tells the ExtendScript Toolkit (or a file handler) which application runs the script. The host executes a script in the application's JavaScript engine; a separate **ScriptingListener** records user actions as `executeAction`/`ActionDescriptor` code.

The object model, rooted at the global `app` (`Application`), surfaces:

| Object | Role |
|---|---|
| `Application` (`app`) | Root. `activeDocument`, `documents`, `foregroundColor`, `backgroundColor`, `currentTool`, `preferences`, `notifiers`, `fonts`, `colorSettings`, `version`, `build`. Methods: `open`, `batch`, `doAction`, `executeAction`, `executeActionGet`, `charIDToTypeID`, `stringIDToTypeID`, `typeIDToCharID`, `typeIDToStringID`, `showColorPicker`, `runMenuItem`, `purge`, `refresh`, `beep`, progress APIs. |
| `Document` / `Documents` | Canvas, color mode, resolution, layers, channels, paths, history, layer comps, selection; save/open options. |
| `Layer` / `ArtLayer` / `LayerSet` / collections | Pixel/text/adjustment/fill layers, groups, opacity, blend mode, visibility, masks, styles, smart objects, `TextItem`, `LayerComp`. |
| `Channel` / `Channels` | Color and alpha channels. |
| `Selection` | Current selection region and operations. |
| `PathItem` / `SubPathItem` / path points | Vector paths (`PathPointInfo` etc.). |
| `HistoryState` / `HistoryStates` | History states and snapshots. |
| `ActionDescriptor`, `ActionList`, `ActionReference` | Low-level "ActionManager" event data; the CS6 equivalent of modern `batchPlay`. |
| `Notifier` / `Notifiers` | Event subscriptions configured in the **Scripts Events Manager**. |
| `Preferences`, `DocumentInfo`, `XMPMetadata`, save/open option classes | Settings and per-format options. |

**Actions and events.** `app.doAction(name, actionSet)` plays a recorded action from the Actions panel. `app.executeAction(eventID, descriptor, dialogMode)` (and `executeActionGet`) play raw ActionManager events. Four-char codes are converted with `charIDToTypeID` / `typeIDToCharID`, and string IDs with `stringIDToTypeID` / `typeIDToStringID`. `Notifier`s (managed via the Scripts Events Manager) fire scripts on events. This ActionDescriptor layer is the mechanism scripts use to reach behavior not covered by the typed DOM.

**ScriptUI** provides dialogs and controls (`Window`, panels, buttons, edit fields, lists, tabs, sliders, radio/checkbox, tree, drawing, layout managers, event callbacks) from JavaScript.

Official references: the *Adobe Photoshop CS6 JavaScript Scripting Reference* (object model and enumerations), the *Photoshop CS6 Scripting Guide* (concepts, installation, `#target`), and Adobe's *JavaScript Tools Guide* (ExtendScript language, File/Folder, Socket, ScriptUI, XML/XMP). See **Sources** for the exact URLs fetched.

### What Kooka Pictura implements

1. A new, stable, versioned C ABI for native plug-ins (in-process, trusted) plus a Rust trait path for in-tree plug-ins.
2. A JavaScript scripting host (QuickJS) with an Kooka Pictura DOM, a restricted ExtendScript compatibility shim, and an action-descriptor command layer.
3. Explicit statement that Adobe `.8bf`/`.8li`/etc. binaries are **not** loadable.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Filter > <plugin category>` | menu | — | Filter-type plug-ins appear under Filter, grouped by PiPL `Category`. |
| `File > Import > <name>` | menu | — | Import/acquire plug-ins. |
| `File > Export > <name>` | menu | — | Export plug-ins. |
| `File > Save As` / `Open` format list | dialog option | — | File-format plug-ins contribute types to Open/Save dialogs. |
| `File > Automate > <name>` | menu | — | Automation plug-ins. |
| `File > Scripts > <name>` | menu | — | Installed `.jsx` scripts; `Browse...` runs an arbitrary script. |
| `File > Scripts > Scripts Events Manager` | dialog | — | Maps events to scripts (notifiers). |
| Actions panel | panel | F9 | `doAction` source; can trigger scripts. |
| Status bar progress | widget | — | `doProgress`/`doProgressTask` display. |
| Plug-ins manager (new) | dialog | — | List/disable/enable Kooka Pictura native plug-ins (replaces Photoshop's folder scan; see Edge cases). |
| Script console / error panel (new) | panel | — | Script output and tracebacks (ExtendScript Toolkit equivalent). |

## Parameters & ranges

This spec defines interfaces, not user-adjustable image parameters. The user-visible control surfaces are the plug-in/script metadata fields.

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Plug-in `Kind` | enum | — | filter, import, export, format, automation, colorPicker, general | Capability bit derived from Kind. |
| Plug-in `Name` | UTF-8 string | — | 1–47 bytes | CS6 PiPL name limit; enforced for UI parity. |
| Plug-in `Category` | UTF-8 string | `Kooka Pictura Plugins` | free text | Menu grouping. |
| ABI major/minor | u32 / u32 | `1` / `0` | host-enforced | Major mismatch = refuse load. |
| `struct_size` | usize | — | must equal or exceed host minimum | Forward-compatible extension via trailing fields. |
| Capability bitset | u64 | plugin-declared | see ABI sketch | Host intersects with granted set. |
| Script timeout | duration | `30 s` | `0` = unlimited | QuickJS interrupt handler / Lua hook. |
| Script memory limit | bytes | `64 MiB` | `0` = unlimited | QuickJS `set_memory_limit`. |
| Script stack limit | bytes | `256 KiB` | — | QuickJS default `256*1024`; `set_max_stack_size`. |
| Module allowlist | path list | empty (deny all) | per-script | Controls `import`/`require` file access. |
| Dialog mode | enum | `ALL` | `ALL`, `ERROR`, `NO` | Mirrors `app.displayDialogs`. |

## Algorithms & pipeline

### Proposed Rust-native plug-in ABI

Design goals: stable across Rust and compiler versions, usable by non-Rust plug-ins in principle, forward-compatible without breaking existing binaries, no panics across the boundary, capability negotiation, and optional isolation.

Rules:

1. **C ABI only.** Every exported symbol is `extern "C"`; every shared type is `#[repr(C)]` with fixed-width integers. No Rust `String`, `Vec`, trait objects, or `Option`-references cross the boundary (Rust has no stable ABI; see Rustonomicon FFI).
2. **One versioned entry symbol.** The loader resolves exactly one symbol, `kookapictura_plugin_entry_v1`, and reads a versioned struct from it. Future incompatible revisions add `_v2` rather than mutate `_v1`.
3. **Version + size header.** The struct begins with `abi_major`, `abi_minor`, `struct_size`. Host rejects on major mismatch, warns on minor mismatch, and only reads fields within `min(host_size, plugin_size)`. New optional fields are appended after a sentinel field and are null/zero when absent.
4. **Capability negotiation.** Plugin declares desired capabilities; host returns the granted set (never a superset of requested or of host-supported). A plugin whose required capability is denied fails to load with a diagnostic.
5. **Opaque handles over pointers.** The DOM is referenced by `OpDocId`/`OpLayerId` (generational indices), never raw host pointers; all access goes through host-owned function pointers in `OpHostApi`. This keeps the plugin independent of internal layouts.
6. **Error codes, not exceptions.** Every call returns a status enum + an optional host-owned error string. Panics must not cross the boundary: exported functions wrap bodies in `catch_unwind` and return an error; `panic = "abort"` is *not* required of plug-ins but is recommended (C-`unwind` ABI is documented as the future path).
7. **Allocator contract.** Memory created by host is freed by host functions and vice versa; pixel buffers are passed as a vtable + `host_ctx` (or as exported `alloc`/`free` pairs) so neither side mixes allocators.
8. **No unload by default.** After `libloading::Library::load`, the `Library` is leaked (or the plugin is pinned) because unloading a Rust `cdylib` with lingering TLS/statics is not reliably safe; hot-*reload* is approximated by loading a new version and draining the old.

### ABI sketch

```text
/* kookapictura_plugin.h — stable C ABI, revision 1.
 * Types are fixed-width. All functions are extern "C".
 * The host never passes an internal Rust type across this boundary. */

#include <stdint.h>
#include <stddef.h>

#define OP_ABI_MAJOR 1u
#define OP_ABI_MINOR 0u
#define OP_ENTRY_SYMBOL_1 "kookapictura_plugin_entry_v1"

typedef enum {
    OP_OK = 0,
    OP_ERR_VERSION      = 1,
    OP_ERR_CAPABILITY   = 2,
    OP_ERR_INVALID_ARG  = 3,
    OP_ERR_CANCELLED    = 4,
    OP_ERR_HOST         = 5,
    OP_ERR_PANIC        = 6
} OpStatus;

typedef struct { const uint8_t* ptr; size_t len; }                OpStr;   /* UTF-8, not NUL-terminated */
typedef struct { int32_t x, y, w, h; }                            OpRect;
typedef uint64_t OpDocId;
typedef uint64_t OpLayerId;
typedef uint64_t OpPluginCtx;

/* Capability bitset. Plug-in declares `wanted`; host fills `granted`. */
enum {
    OP_CAP_FILTER          = 1ull << 0,   /* transform pixels of a layer/selection */
    OP_CAP_IMPORT          = 1ull << 1,
    OP_CAP_EXPORT          = 1ull << 2,
    OP_CAP_FORMAT_READ     = 1ull << 3,
    OP_CAP_FORMAT_WRITE    = 1ull << 4,
    OP_CAP_AUTOMATION      = 1ull << 5,
    OP_CAP_PIXEL_READ      = 1ull << 6,
    OP_CAP_PIXEL_WRITE     = 1ull << 7,
    OP_CAP_DOC_MUTATE      = 1ull << 8,   /* create/delete layers, etc. */
    OP_CAP_FILESYSTEM      = 1ull << 9,   /* explicit opt-in to file IO */
    OP_CAP_NETWORK         = 1ull << 10   /* denied by default */
};

/* Host-owned function table handed to the plug-in at negotiate().
 * All pointers may be NULL if the corresponding capability is not granted. */
typedef struct OpHostApi {
    uint32_t abi_major;          /* == OP_ABI_MAJOR */
    uint32_t abi_minor;
    size_t   struct_size;
    void*    host_ctx;

    OpStatus (*doc_get_pixels)(void* host_ctx, OpDocId doc, OpLayerId layer,
                               OpRect roi, void** out_buf, size_t* out_len,
                               size_t* out_row_bytes, uint32_t* out_format);
    OpStatus (*doc_commit_pixels)(void* host_ctx, OpDocId doc, OpLayerId layer,
                                  OpRect roi, const void* buf, size_t len,
                                  size_t row_bytes, uint32_t format);
    OpStatus (*doc_create_layer)(void* host_ctx, OpDocId doc, OpStr name,
                                 OpLayerId* out_layer);
    OpStatus (*doc_delete_layer)(void* host_ctx, OpDocId doc, OpLayerId layer);
    OpStatus (*undo_begin)(void* host_ctx, OpStr label);      /* transaction */
    OpStatus (*undo_commit)(void* host_ctx);
    OpStatus (*undo_abort)(void* host_ctx);
    void     (*log)(void* host_ctx, OpStr message);
    /* Append new host functions here; struct_size guards their visibility. */
} OpHostApi;

/* Request/result passed to process(). Kept minimal and extensible. */
typedef struct OpProcessRequest {
    size_t   struct_size;
    uint32_t selector;    /* per-kind opcode, cf. filterSelectorStart/Continue */
    OpDocId  doc;
    OpLayerId layer;
    OpRect   roi;
    void*    params;      /* plug-in-defined parameter block */
} OpProcessRequest;

typedef struct OpProcessResult {
    size_t   struct_size;
    uint32_t flags;       /* e.g. OP_RESULT_NEEDS_ANOTHER_PASS */
    OpStr    message;
} OpProcessResult;

/* The single exported plug-in descriptor. */
typedef struct OpPluginApi {
    uint32_t     abi_major;      /* OP_ABI_MAJOR */
    uint32_t     abi_minor;      /* OP_ABI_MINOR */
    size_t       struct_size;
    OpStr        id;             /* reverse-DNS, stable */
    OpStr        name;           /* display name, <= 47 bytes */
    OpStr        category;
    OpStr        version;        /* semver string */
    uint64_t     wanted_caps;    /* declared by plug-in */
    uint64_t     granted_caps;   /* filled by host in negotiate() */

    OpStatus (*negotiate)(const OpHostApi* host, uint64_t host_caps);
    OpStatus (*init)(const OpHostApi* host, OpPluginCtx* out_ctx);
    OpStatus (*process)(OpPluginCtx ctx, const OpProcessRequest* req,
                        OpProcessResult* res);
    void     (*shutdown)(OpPluginCtx ctx);
} OpPluginApi;

#if defined(_WIN32)
#  define OP_EXPORT __declspec(dllexport)
#else
#  define OP_EXPORT __attribute__((visibility("default")))
#endif

/* Exactly one such symbol; the loader asks for this name and nothing else.
 * Returns NULL if the plug-in cannot initialise. */
OP_EXPORT const OpPluginApi* kookapictura_plugin_entry_v1(void);
```

**Load and negotiation sequence** (host side):

```text
1. dlopen path via libloading::Library::new(path)
2. sym = lib.get::<extern "C" fn() -> *const OpPluginApi>(b"kookapictura_plugin_entry_v1\0")
   - symbol absent -> "not an Kooka Pictura plug-in", skip
3. api = sym(); if api.is_null() -> skip
4. if api->abi_major != OP_ABI_MAJOR -> refuse, log versions
5. verify api->struct_size >= offsetof(OpPluginApi, negotiate)+sizeof(fn)
6. api->negotiate(&host_api, host_caps)
   - if (api->wanted_caps & ~host_caps) contains a *required* bit -> refuse
   - api->granted_caps = api->wanted_caps & host_caps
7. api->init(&host_api, &ctx)
8. register (kind from granted_caps) in the plug-in registry; populate menus
9. on shutdown: api->shutdown(ctx); drop host references; do NOT dlclose
```

**Symbol versioning:** the entry symbol itself carries the revision (`_v1`). Within a revision, compatibility is governed by the `abi_major`/`abi_minor`/`struct_size` header: host reads only the prefix it knows and treats absent trailing function pointers as unsupported. A plug-in must not reorder or resize existing fields; it may only append after the sentinel.

**Rust in-tree vs dynamic C ABI.** Two registration paths exist but one execution path:

- In-tree plug-ins implement `trait Plugin` and are registered in a compile-time registry (`inventory`-style or an explicit `Vec`). They get direct access to typed core APIs and are always available.
- Dynamic plug-ins implement the C ABI above and are adapted into `trait Plugin` by a thin `CAbiAdapter`.
- Both are dispatched through the same `PluginRegistry`, so UI and command handling are identical.
- Intra-Rust dynamic plug-ins *may* additionally use `abi_stable` (load-time layout checking, FFI-safe `RStr`/`RVec`, `sabi_trait`) in a future revision, but it is **not** the primary contract: `abi_stable` pins compatible Rust/`abi_stable` versions and offers no cross-language binding, so the C ABI remains the stable surface.

**Sandboxing.** Trust mirrors Photoshop: plug-ins are locally installed and trusted, so the default is in-process with capability checks (filesystem/network opt-in). For untrusted plug-ins the spec reserves an out-of-process host (separate process, shared-memory pixel transport, crash isolation) and/or a WASM module path (`wasmtime`) — neither exists in CS6 and both are Non-goal for initial parity. `libloading`/`abi_stable` provide no sandbox of their own (community analysis: full host access).

### Proposed scripting host

**Engine choice.** Candidates evaluated from their own docs:

| Engine | Rust crate | Language | Sandbox primitives | Fit |
|---|---|---|---|---|
| QuickJS | `rquickjs` | JavaScript ES2025 | `set_memory_limit`, `set_max_stack_size`, `set_interrupt_handler` (timeout), module loader control, custom allocator, `parallel`/`futures` features | Closest to ExtendScript (both JS); best migration story. |
| Lua | `mlua` | Lua 5.1–5.5 / Luau | debug hooks/`HookTriggers`, `VmState`, `StdLib` subsetting, `send` feature | Mature and fast; DOM ergonomics weaker for JS users. |
| Rhai | `rhai` | Rhai (JS-like) | sandboxed by design, immutable `Engine`, operation/recursion/progress limits, opt-in `unchecked` disables safety | Best safety/simplicity; not JS; no E4X/DOM parity. |

**Recommendation:** primary engine **QuickJS via `rquickjs`**. Rationale: existing CS6 scripts are JavaScript; QuickJS is a small, embeddable, no-ICU engine with explicit memory/stack/interrupt controls and ES module loading; it gives the shortest path to running `.jsx` with minimal rewriting and native JSON. Rhai is the fallback if we decide compatibility with JS is not worth it; Lua is not preferred.

**Exposed object model (proposed).** A subset, modeled on CS6 names, bound as QuickJS classes:

- `app` → `Application` (`activeDocument`, `documents`, `foregroundColor`, `backgroundColor`, `preferences`, `version`, `displayDialogs`).
- `Document`, `Documents`, `Layer`/`ArtLayer`/`LayerSet`, `Layers`/`ArtLayers`/`LayerSets`, `Channel`/`Channels`, `Selection`, `PathItem`, `HistoryState`/`HistoryStates`, `LayerComp`, `TextItem`, `ActionDescriptor`/`ActionList`/`ActionReference`.
- Methods: `app.open`, `app.doAction`, `app.executeAction`, `app.executeActionGet`, `app.charIDToTypeID`, `app.stringIDToTypeID` (and inverses), `app.runMenuItem`, `app.showColorPicker`, `document.saveAs`/`exportDocument`, progress APIs.
- **Every mutation routes through the command/action-descriptor layer** (the same layer the UI and native plug-ins use), so scripting, actions, and plug-ins share one undo/command path.

**Do we emulate ExtendScript?** Partially. We implement a **compatibility shim** that maps the documented ExtendScript DOM subset used by the majority of scripts (`app`, `Document`, `Layer`, `executeAction`/`ActionDescriptor`, `File`/`Folder`, `alert`/`prompt`) onto the Kooka Pictura command layer. We do **not** promise full emulation:

- E4X (`XML` literals) and operator overloading are ES3-era ExtendScript extensions absent from modern engines; scripts using them must be rewritten or run under a transpiler that does not depend on E4X.
- ExtendScript's `File`/`Folder`/`Socket`/`ExternalObject` classes are replaced by sandboxed equivalents or omitted (no `ExternalObject` native loading).
- ScriptUI is not reproduced; dialogs can be emitted through the Qt host or a minimal ScriptUI-compatible shim over Qt widgets.
- Adobe confirms ExtendScript stays ES3 and that modern UXP is *not* a drop-in replacement; the same is true here.

**Sandboxing.** Default-deny: no filesystem or network unless the user grants it per script; module imports restricted to an allowlist; QuickJS memory limit, stack limit, and an interrupt-handler wall-clock timeout (default 30 s); output captured to the script console; a failed script aborts its transaction and leaves the document unchanged.

**Migration path.**

1. Phase 1 — engine + DOM subset: open/save, layer properties, actions/`doAction`, selector filters.
2. Phase 2 — action descriptors (`executeAction`/`charIDToTypeID`) and `Notifier`s; Script Events Manager.
3. Phase 3 — compatibility shim for common `.jsx`, `#target`, `#include`; ship a script converter that rewrites E4X-free ES3 to the shim.
4. Phase 4 — ScriptUI shim and `File`/`Folder` sandbox.
5. Phase 5 — Rhai/embedded-Rust documented alternative for users who prefer a sandboxable non-JS language.

### Why not load Adobe plug-ins

`Adobe binary compatibility = Non-goal (Linux)`. Documented reasons: PE/Mach-O + platform resource sections; MSVC/Apple toolchain-specific ABI and mangling; `pipltool`/`Rez`/`.rc` resource pipeline with no Linux equivalent; Carbon-era types (`Str255`, `Fixed`, `Rect`); the proprietary `SPBasicSuite` host interface; and the SDK license. Kooka Pictura reads and writes Adobe *file formats* (PSD/PSB) but does not execute Adobe *binary* plug-ins.

## Rust module mapping

- `crate::plugin::abi` — `#[repr(C)]` mirror of the C ABI structs (`OpPluginApi`, `OpHostApi`, `OpProcessRequest/Result`, enums), plus compile-time `const_assert` size/offset checks and the `OP_ENTRY_SYMBOL_1` constant.
- `crate::plugin::loader` — `libloading::Library` wrapper; resolve `kookapictura_plugin_entry_v1`, validate ABI header, keep `Library` alive for the process lifetime (no `dlclose`). Returns `LoadedPlugin`.
- `crate::plugin::registry` — `PluginRegistry` keyed by `id`; capability resolution against `HostCaps`; kind → menu/category mapping; enable/disable state persisted in preferences.
- `crate::plugin::host_api` — implements `OpHostApi`: translates `OpDocId`/`OpLayerId` generational handles into `DocumentModel` borrows, starts/commits undo transactions, copies pixel buffers, routes logs.
- `crate::plugin::Plugin` trait (in-tree) + `CAbiAdapter` (dynamic) — common dispatch interface; `ProcessCtx`, `ProcessOutcome`.
- `crate::plugin::sandbox` — capability enforcement; reserved out-of-process/WASM backend behind a trait.
- `crate::command` / `crate::action` — the shared command bus + ActionDescriptor/ActionList/ActionReference model; the single mutation entry point for UI, scripts, and plug-ins. Host API pixel mutations and script DOM mutations both compile to commands here.
- `crate::script::engine` — `rquickjs::Runtime` + `Context`; wraps a single-threaded `Runtime` behind a dedicated script thread; applies `set_memory_limit`, `set_max_stack_size`, `set_interrupt_handler`, custom `set_loader` allowlist.
- `crate::script::dom` — QuickJS `JsClass` wrappers (`Application`, `Document`, `Layer`, ...) mapped to the command bus; `IntoJs`/`FromJs` conversions.
- `crate::script::actions` — `executeAction`, `doAction`, `charIDToTypeID`/`stringIDToTypeID` and inverses; `ActionDescriptor` serialize/deserialize.
- `crate::script::compat` — ExtendScript shim: `#target`/`#include` handling, `File`/`Folder`/`alert`/`prompt`, legacy name aliases, E4X diagnostics.
- `crate::script::sandbox` — permission model, timeouts, module allowlist, output capture.
- `crate::script::notifiers` — Script Events Manager: event subscription table, firing from `crate::command`'s event stream.

## Qt6 component mapping

- `PluginManagerDialog` — lists native plug-ins (id, name, category, version, granted capabilities), enable/disable, load errors. Table model `PluginListModel`.
- `ScriptsMenu` (built into `QMenuBar`) — `File > Scripts > <installed>` + `Browse...` + `Scripts Events Manager...`; backed by `ScriptRegistryModel`.
- `ScriptEventsManagerDialog` — event ↔ script mapping editor (CS6 parity).
- `ScriptConsoleDock` — output, errors, timings; buttons Run/Stop; a `QPlainTextEdit` with a run/stop toolbar (replaces ExtendScript Toolkit's minimal console).
- `ScriptSecurityDialog` — per-script filesystem/network permission prompt; model `ScriptPermissionModel`.
- `ProgressProxy` — `QProgressDialog` bridge for `doProgress`/`doProgressTask` and native plug-in progress callbacks, with cancel → `OP_ERR_CANCELLED`.
- `PluginErrorBanner` — non-modal warning when a plug-in fails ABI/capability checks.

Widgets (not QML) are proposed for the manager/console because they are document-modal utilities over a `QMainWindow` and must match the CS6 docked-panel feel; QML is reserved for the script-authored dialog shim where layout flexibility matters.

## Data-model impact

- **New node: `PlugInRegistration`** (runtime, not persisted in PSD): `id`, `kind`, `capabilities`, `source` (builtin | c-abi path), `enabled`.
- **New node: `ScriptEventBinding`** (persisted in preferences): `event_id`, `script_path`, `enabled`, `notify` flag — mirrors CS6 notifiers.
- **Command/ActionDescriptor model.** `ActionDescriptor` (`std::collections` map of `ActionKey` → `ActionValue`) and `ActionList` are first-class in-memory types shared by the script DOM, `executeAction`, and native plug-in host API. This is the serialization seam for actions.
- **Handles.** `OpDocId`/`OpLayerId` are generational indices into the document arena; they never leak host addresses and are invalidated on close/delete. Plug-ins are required to re-fetch after a mutation.
- **Undo granularity.** `undo_begin`/`undo_commit` bracket one logical plug-in operation or one top-level script invocation into a **single** history state labeled with the plug-in/script name. Nested `doProgressSubTask` calls fold into the parent transaction. A script that emits many `executeAction`s defaults to one undo step per top-level script run (configurable), matching the user expectation that "run a script = one undo".
- **Undo record shape.** `HistoryRecord::PluginOp { plugin_id, selector, roi, layer, before_hash, after_hash }` and `HistoryRecord::ScriptOp { script_path, command_ids }`. Pixel payloads are stored by the existing history/tile mechanism, not duplicated by the plug-in ABI.
- **Serialization.** Plug-in/script metadata is **not** written into PSD/PSB. ActionDescriptors serialize to the existing action XML/JSON representation when exported as actions; script paths live in preferences. XMP is untouched by plug-in registration.
- **File-format plug-ins** integrate with the format read/write pipeline via `OP_CAP_FORMAT_READ/WRITE`; they receive/return `FormatRecord`-style geometry and plane metadata mapped to the internal image format.

## Edge cases

- **ABI/version mismatch:** major mismatch refuses load with a diagnostic; minor mismatch loads and hides unknown trailing fields. Missing entry symbol → skip silently during directory scan, log at debug.
- **Panic across FFI:** undefined behavior in Rust; exported functions wrap in `catch_unwind` and translate to `OP_ERR_PANIC`; a panicking plug-in is disabled and reported, the host survives. `C-unwind` is noted as the future ABI but not relied on.
- **Thread safety:** the host API is **not** thread-safe (Adobe documents the same). Calls are serialized on the document thread; plug-ins requiring background work must use host-provided worker primitives introduced later.
- **Allocator mismatch:** host allocates/frees host buffers; plugin buffers returned via a host `free` callback; never `free()` a Rust `Box` across the boundary.
- **Unload/hot-reload:** `dlclose` is avoided (leaked/pinned `Library`) to prevent use-after-unload from TLS/statics; "hot reload" loads a new library version and drains the old on next idle. Memory grows per reload — documented ceiling.
- **Pixel formats across the boundary:** 8/16/32-bit, Gray/RGB/CMYK/Lab, planar vs interleaved, row padding. The ABI carries an explicit `uint32_t format` code + `row_bytes`; no implicit conversion; CMYK/Lab and 32-bit float supported via format codes; endianness fixed little-endian for buffer payloads.
- **Empty/1-px documents and zero-area ROI:** reject `w == 0 || h == 0` with `OP_ERR_INVALID_ARG`; 1×1 allowed.
- **PSB/huge documents:** ROI-based access only; a plug-in may not ask for the whole canvas if it exceeds a configured tile budget; `int32_t` ROI fields are a known limitation near 300,000 px (see Open questions).
- **GPU unavailable:** plug-ins and scripts never assume Mercury/GPU; host API pixel operations fall back to CPU. No GPU handles cross the ABI.
- **Script infinite loop / stack blow-out / memory bomb:** interrupt-handler timeout, stack limit, memory limit; on trip, abort the script's transaction and surface a traceback. Rhai's documented DoS vectors (strings, arrays, closures, deep recursion, self-referencing modules) are the checklist if Rhai is adopted.
- **Script file/module access:** default-deny; symlink/`..` traversal rejected; no `ExternalObject` native loading.
- **Reentrancy:** a script that triggers an event which fires a notifier that runs a script must be depth-limited; notifier firing is deferred to the command loop (no reentrant `executeAction`).
- **Event ordering:** notifiers observe committed commands only; a notifier must not mutate the transaction it is observing.
- **Dialog mode:** `displayDialogs = NO` suppresses modal dialogs for batch/automation; a plug-in requesting a dialog under `NO` gets `OP_ERR_CANCELLED` or a default.
- **Locale/encoding:** plug-in `Name` is UTF-8; PiPL's 47-char limit is enforced in bytes for menu parity. Paths use `OpStr` UTF-8, not `Str255`.

## Parity acceptance criteria

1. Given a file-format plug-in loaded from the plug-in directory, opening a file whose type it claims appears in `File > Open` and delegates to the plug-in's `process` with a `formatSelectorReadStart`-equivalent request.
2. Given a filter plug-in declaring `OP_CAP_FILTER` and requesting `OP_CAP_NETWORK`, negotiation grants filter and denies network; attempting network access returns `OP_ERR_CAPABILITY`.
3. Given a plug-in built against ABI major 2 and a host at major 1, load is refused and the manager shows a version-mismatch entry.
4. Given a plug-in whose exported entry panics, the host reports `OP_ERR_PANIC`, disables the plug-in, and the document and host remain usable.
5. Given a `.jsx` script `#target photoshop` that sets `app.activeDocument.activeLayer.opacity = 50`, running it changes the active layer opacity to 50 and creates exactly one history state named after the script.
6. Given a script calling `app.charIDToTypeID("Opn ")` and `app.executeAction(...)` for a documented event, the resulting document mutation matches the UI command within existing per-command tolerances.
7. Given `app.displayDialogs = DialogModes.NO`, a script that would show a modal dialog completes without UI and without blocking.
8. Given an infinite-loop script, the host interrupts it within the configured timeout (±1 s), the transaction is rolled back, and no history state is added.
9. Given a script requesting filesystem access without a grant, `File`/module reads fail and the denial is logged; with a grant, reads inside the allowed roots succeed and outside them fail.
10. Given a native plug-in started with `undo_begin` and aborted, the document is byte-identical to before the call (pixel hash equality) and no history state exists.
11. Given 8/16/32-bit and CMYK/Lab documents, a round-trip filter plug-in (read → identity → write) produces output within one LSB (integer modes) or `1e-5` relative (32-bit float) of the input.
12. Given a `.8bf` Adobe binary placed in the plug-in directory, it is not loaded, no crash occurs, and the manager reports "unsupported format (Adobe binary compatibility is a non-goal)".

## Sources

- `https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/cpp-pluginsdk` — entry points `AutoPluginMain`/`PluginMain`, `SPBasicSuite`, `AcquireSuite`, `PIActionDescriptor`, suites, PiPL component name.
- `https://en.wikipedia.org/wiki/Photoshop_plugin` — plug-in type table (extensions, Mac type codes), PiPL/API-guide citation, SDK access history.
- `https://raw.githubusercontent.com/sonictk/ps_cpp_recipes/master/docs/index.html` — community tutorial: entry-point signatures, `FilterRecord`/`ExportRecord` fields, `Str255`/`Fixed`, PiPL `.r` resource, `pipltool`/`cnvtpipl` build pipeline, `sSPBasic`, `kPSDoIt`, host API not thread-safe, `PluginMain`/`AutoPluginMain` full sample.
- `https://ae-plugins.docsforadobe.dev/intro/pipl-resources` — PiPL definition, properties (Kind/Name/Category/Entry Point/version/flags), big-endian byte order, per-platform resource compilation.
- `https://deepwiki.com/webmproject/WebPShop/2.1-entry-point-and-selector-dispatch` — `PluginMain` selector dispatch, `Data` persistence, `PSChannelPortsSuite`, `formatSelector*` sequences, `FormatRecordPtr` usage, exception wrapping.
- `https://docs.rs/abi_stable/latest/abi_stable` — Rust-to-Rust FFI, `StableAbi`, prefix types, `sabi_trait`, load-time layout checking, no-unload support.
- `https://docs.rs/libloading/latest/libloading/` — `Library::new`, `Symbol`, `extern "C"`, lifetime safety.
- `https://nullderef.com/blog/plugin-abi-stable/` — `abi_stable` vs raw C ABI, versioning, no sandbox, panic-across-FFI handling (`AbortBomb`/`catch_unwind`), `C-unwind`, type conversion cost, thread-safety of `dlerror`, performance of dynamic loading.
- `https://doc.rust-lang.org/nomicon/ffi.html` — `extern "C"`, `#[repr(C)]`, `crate-type = ["cdylib"]`, `#[no_mangle]`, callbacks, opaque structs, FFI and unwinding, `C-unwind`.
- `https://rhai.rs/book/safety/index.html` — Rhai DoS vectors and "Don't Panic" guarantee.
- `https://rhai.rs/book/safety/sandbox.html` — Rhai sandboxing, immutable `Engine`.
- `https://docs.rs/rquickjs/latest/rquickjs` — QuickJS binding overview, Runtime/Context, module loader, `futures`/`parallel`.
- `https://docs.rs/rquickjs/latest/rquickjs/runtime/struct.Runtime.html` — `set_memory_limit`, `set_max_stack_size` (default `256*1024`), `set_interrupt_handler`, `set_loader`, `memory_usage`, `run_gc`.
- `https://bellard.org/quickjs/quickjs.html` — QuickJS features, ES2025 support, `JS_SetMemoryLimit`, `JS_SetMaxStackSize`, `JS_SetInterruptHandler`, C modules, no-ICU.
- `https://docs.rs/mlua/latest/mlua` — Lua bindings, `UserData`, `StdLib`, debug hooks/`HookTriggers`, `VmState`, `send`/`Send` model.
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/Application.html` — CS6 `Application`/`app` surface (properties and methods list), `executeAction`, `doAction`, `charIDToTypeID`, notifiers.
- `https://theiviaxx.github.io/photoshop-docs/scripting.html` — Photoshop scripting reference mirror index and class list.
- `https://extendscript.docsforadobe.dev/` and `https://extendscript.docsforadobe.dev/introduction/extendscript-overview/` — ExtendScript overview, ScriptUI, File/Folder, BridgeTalk, ExternalObject, XML, XMP.
- `https://extendscript.docsforadobe.dev/extendscript-tools-features/preprocessor-directives/` — `#include`, `#includepath`, `#script`, `#strict`, `#target`, `#targetengine`.
- `https://manuals.plus/m/4d2aa1ad9d53272237bc296b264d3168e6a639031a29d186fbcbfccb12c52c20` — *Adobe Photoshop CS6 JavaScript Scripting Reference* (231 pp.) overview: Application/Document/Layer model, file-format save options, script execution.
- `https://community.adobe.com/t5/photoshop-ecosystem-ideas/p-ps-scripting-extendscript-support-for-ecmascript-6th-version/idc-p/12834027` — Adobe/community confirmation: ExtendScript is ECMA-262 **v3** + E4X, no ES6; UXP is not a drop-in replacement.

Community/archival sources (not Adobe-hosted): Wikipedia, ps_cpp_recipes, docsforadobe, theiviaxx, deepwiki/WebPShop, manuals.plus, Adobe community forums. Marked where used.

## Open questions

- **Exact `PIProperty` semantics.** Adobe's API guide describes PiPL property entries; the precise type/struct named `PIProperty` was not retrieved. Resolution: obtain the Photoshop CS6 SDK headers (`PIGeneral.h`/`PIProperty`) or the API-guide PDF and record the exact values; if unavailable, keep the Kooka Pictura metadata model independent of it.
- **Engine decision is provisional.** QuickJS (`rquickjs`) is recommended but not ratified. Resolve with a benchmark/binding spike covering DOM ergonomics, startup latency, lifecycle with Qt, and packaging on Linux (rquickjs prebuilt bindings list musl/gnu targets; other targets need `bindgen`).
- **ExtendScript compatibility depth.** Which `.jsx` subset must run unmodified? Resolve by surveying real CS6 scripts (e.g. Adobe sample scripts, common Photoshp script libraries) and measuring E4X/ScriptUI/`File` usage.
- **E4X and `#include` support.** Whether to implement an E4X shim, transpile, or reject with diagnostics is undecided.
- **Sandbox model.** In-process capability checks (CS6-like) vs out-of-process vs WASM for untrusted plug-ins. Resolve with a threat model and the accessibility/permissions policy in `11-cross-cutting/security-and-sandboxing`.
- **`abi_stable` as optional Rust-to-Rust layer.** Whether to support it in addition to the C ABI, given version pinning and no cross-language bindings. Resolve after the C ABI ships.
- **Hot-reload semantics.** Whether pinning/leaking `Library` is acceptable long-term, and whether an idle drain solves state leaks. Resolve with a load/leak measurement.
- **ROI coordinate width.** `int32_t` fields cap at ~2.1 Gpx; PSB allows 300,000 px per side. Decide whether to add an `OpRect64` variant before ABI v1 freezes.
- **ActionDescriptor serialization format.** Whether to match Adobe's action XML (`.atn`/action file) or define a JSON schema, and whether `.atn` export is in scope.
- **ScriptUI shim scope.** Whether to implement ScriptUI over Qt widgets, expose only a minimal dialog API, or omit it and document rewriting.
- **Script Events Manager persistence and security.** Storing script paths and auto-running on events is an attack surface; resolve with the security policy doc.
- **Native plug-in discoverability.** CS6 scans the plug-ins folder; Kooka Pictura proposes a manager. Whether to also scan a directory (and with what confinement) is open.
- **Locale handling of plug-in `Name`.** CS6's 47-character limit is legacy; confirm whether parity requires the same byte/character limit and how localized names are supplied (no PiPL localization equivalent yet).
- **`.atn`/droplet compatibility.** Whether Kooka Pictura action files interoperate with Adobe `.atn` and droplets, or only with its own format.
