# Mini Bridge

- **Spec ID:** `AUTO-014`
- **Status:** `Draft`
- **Parity tier:** `Non-goal (Linux)` — Mini Bridge depends on Adobe Bridge, a Flash-based CEP extension host, and the proprietary SwitchBoard transport, none of which have a Linux equivalent; a native file browser is proposed instead.
- **New in CS6:** `Changed` — the CS6 Help markets Mini Bridge as a "new Mini Bridge gallery" and the panel is present, but Mini Bridge was introduced in CS5 and its CS6 incarnation was degraded (the Application bar, and with it the launch-Bridge button, was removed). The Help's "new" label conflicts with CS5-era Adobe material; see Open questions.
- **Depends on:** `AUTO-012` (plugin-sdk / extension plug-ins), `10-workflow-io/bridge-and-interop`, `02-ui-ux/workspace-and-docks`, `10-workflow-io/open-and-new`, `00-overview/feasibility-and-non-goals`.

## CS6 behavior

Mini Bridge is a **Photoshop extension panel** that embeds a small file browser for Adobe Bridge assets inside Photoshop, rather than requiring the full standalone Bridge application. In CS6 it is opened from **`Window > Extensions > Mini Bridge`** (the Help also lists `File > Browse in Mini Bridge`); it appears as a panel across the bottom of the workspace. It is a simplified browsing UI: image preview, navigation, and opening/placing assets, without Bridge's refinement features (keyword sorting, metadata editing, web/print output) — a convenience rather than a replacement for full Bridge.

Integration and dependencies (per Adobe and community sources):

- Mini Bridge requires **Adobe Bridge** to be installed and typically **running in the background**. Its two dependencies are **Bridge** and **SwitchBoard**; if either is unavailable it shows "Waiting for Bridge CS6...".
- **SwitchBoard** is a network/protocol layer that connects the Mini Bridge panel to Bridge — the pipeline carrying browsing information between the two processes.
- It is a **Flash-based extension**. CS6/CC 2013 host the CEP 4 infrastructure, which supports the older Flash/ActionScript extension model (the HTML5/JS model was also available and preferred).
- In CS6 the **Application bar was removed** (to reclaim vertical space), which removed the Mini Bridge panel's launch-Bridge button; the panel can therefore appear non-functional or icon-less unless Bridge is already connected. Resetting preferences or restarting Bridge/SwitchBoard was the common workaround.
- Mini Bridge was **removed from Photoshop CC 2014** when Flash technology was dropped from Photoshop; Adobe's position was that Flash/ActionScript extensions had to be rewritten in HTML, and that Mini Bridge is a shared, independent product.
- Bridge itself remains an Adobe Creative Suite application with no Linux build.

Net CS6 reality: a Flash-based CEP panel that only works with Adobe Bridge present on Windows/macOS, and which was flaky and then removed. It is not a native Photoshop feature in any meaningful sense.

### Availability in CS6 (verification)

| Claim | Verdict | Evidence |
|---|---|---|
| Mini Bridge exists in CS6 | **Yes** | CS6 Help What's New and `Window > Extensions > Mini Bridge`. |
| CS6 Help calls it "new" | **Misleading** | Adobe CS5 material ("Use Mini Bridge in CS5 applications") predates CS6. |
| Requires Bridge running | **Yes** | Adobe employee responses; "Waiting for Bridge CS6" reports. |
| Flash-based / CEP 4 | **Yes** | Adobe-CEP archived-versions README (CS6/CC2013 = CEP 4, Flash/ActionScript supported). |
| Launch button present in CS6 | **Degraded** | Application bar removed in CS6; community reports missing button/icons. |
| Removed after CS6 | **Yes, in CC 2014** | Adobe employee statement + Adobe "Spring Cleaning" blog, cited by Adobe. |

### Why Linux non-goal

Mini Bridge cannot be ported in any faithful sense because it is not a Photoshop feature:

1. **Adobe Bridge dependency.** Mini Bridge is a thin panel over Bridge; there is no Bridge on Linux and Kooka Pictura does not use one.
2. **Flash/CEP 4 host.** It runs under Adobe's CEP extension host with the Flash/ActionScript interface model. Kooka Pictura uses Qt6; reimplementing CEP + Flash is absurd and out of scope.
3. **SwitchBoard transport.** A proprietary, network-based IPC protocol between the panel and Bridge; undocumented and unnecessary for a native panel.
4. **Removed downstream.** Adobe itself dropped it in CC 2014, so there is no sustained CS6-upward behaviour to preserve.

Requirement satisfied by a **native file browser panel** (`AUTO-014` replacement), not by emulation.

### Proposed replacement: a native Files/Browser panel

A dockable **Files** panel (working name `FilesPanel`) that covers Mini Bridge's useful subset and more, using core Qt6 + the Rust core:

- Folder navigation (tree + breadcrumb) rooted at a configurable start location (Home, Pictures, or a workspace favorite).
- Thumbnail grid (`QListView` in IconMode) with lazy, cancellable thumbnail generation through the render/format pipeline; preview size control (slider).
- Single-image preview pane and basic EXIF/IPTC display read from the existing metadata layer.
- Open (`File > Open` semantics), Place as Smart Object, and drag-and-drop of a file/selection into the canvas or Layers panel (the Bridge→Photoshop behaviours worth keeping).
- Multi-select and "open selected as layers / load files into stack"-style operations via the existing scripts.
- Sorting and filtering (name, date, type, rating), favorites/bookmarks, and recent folders.
- No cloud, no Adobe Bridge, no network transport; all local filesystem through the protected file-access layer.
- Keyboard-navigable and accessible (the CS6 panel was not).

`File > Browse in Mini Bridge` in CS6 is replaced by `File > Browse...` (or the panel) opening files through the OS/portal file dialog, and `File > Open` remains unchanged.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Window > Extensions > Mini Bridge` | menu/panel | — | CS6 entry; non-goal, replaced by `Window > Files`. |
| `File > Browse in Mini Bridge` | menu | — | CS6; replaced by the Files panel / `File > Open`. |
| `Window > Files` (new) | menu/panel | — | Proposed native replacement. |
| Files panel — folder tree | panel control | — | Navigation; favorites/bookmarks. |
| Files panel — thumbnail grid | panel control | — | Lazy thumbs; size slider; multi-select. |
| Files panel — preview/info | panel control | — | Image preview + metadata readout. |
| Files panel — context menu | context menu | — | Open, Open With, Place as Smart Object, Reveal in file manager, Add to favorites. |
| Canvas drag-and-drop | interaction | — | Drop a file/selection to open or place. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Start location | path | Home | Home, last folder, favorite | Persisted preference. |
| Thumbnail size | int (px) | 128 | 64–512 | Slider; drives cache key. |
| Thumbnail cache limit | bytes | 256 MiB | 0 = unlimited | LRU eviction. |
| Sort key | enum | name | name, date modified, size, type, rating | Asc/desc. |
| Filter | string | empty | glob/regex or tag | Live filter. |
| Preview pane | bool | on | on/off | Show/hide. |
| Metadata fields | set | EXIF size/mode | any supported | Read-only in v1. |
| Drop action | enum | Open | Open, Place as Smart Object, Add as Layer | Modifier keys override. |
| Favorites | path list | empty | user-managed | Sidebar entries. |

## Algorithms & pipeline

### Thumbnail and preview generation

```text
request(folder):
  entries = read_dir(folder) filtered/sorted on background thread
  for each entry:
      emit placeholder row immediately
      enqueue thumbnail job (cancellable, deduped by (path, mtime, size, thumb_size))
thumbnail_worker (background, bounded pool):
  decode at reduced resolution using formats::thumbnail(path)   # avoids full decode where possible
  cache by key; evict LRU over limit
  post result to the model on the UI thread
preview(path):
  load via formats::open + render to a QImage/QPixmap at pane size
drag_out(path):
  provide text/uri-list; drop target decides Open vs Place vs Add-as-Layer
```

All heavy work is off the UI thread; jobs are cancelled when the folder changes. Nothing here depends on Bridge, Flash, or a network.

### Why not emulate Mini Bridge

The panel's own logic (browse, preview, open) is small; the bulk of Mini Bridge is protocol glue to Bridge and Flash. Reimplementing that glue buys nothing and would be deleted immediately, so the replacement is a normal Qt model/view panel over the core's format/metadata APIs.

## Rust module mapping

- `crate::browser` — `FolderModel` (entries, sort/filter), `ThumbnailService` (cancellable job queue + LRU cache), `PreviewService`.
- `crate::browser::favorites` — bookmark store in preferences.
- `crate::formats::thumbnail` / `crate::formats::metadata` — reduced-resolution decode and EXIF/IPTC read (reused, not duplicated).
- `crate::document::place` — Smart Object / Add-as-Layer commands for drops.
- `crate::script` — hooks for "load files into stack"-style scripts (reuse built-ins).
- No `crate::bridge` module: Bridge/SwitchBoard are explicitly absent.

## Qt6 component mapping

- `FilesPanel` (`QDockWidget`) containing:
  - `QTreeView` + `QFileSystemModel`-style folder tree (`FolderTreeModel` over `crate::browser`).
  - `QListView` (IconMode, `QAbstractItemModel` = `ThumbnailModel`) with a size slider and multi-select.
  - `QSplitter` layout: tree | grid | preview/info.
- `PreviewPane` (`QLabel`/custom `QWidget`) rendering the selected image.
- `MetadataPane` (`QFormLayout`) showing read-only EXIF/IPTC.
- `FavoritesSidebar` (`QListView`).
- Toolbar: back/forward/up, path breadcrumb, sort combo, filter field, thumbnail-size slider.
- Context menu and drag support (`QDrag` with `text/uri-list`; drop targets on canvas/Layers panel).
- Widgets (not QML): matches the docked-panel model of the CS6 interface and the rest of the UI.

## Data-model impact

- **No document-model change.** The browser reads the filesystem and metadata; opening/placing goes through existing document commands.
- **Preferences additions:** `BrowserStartLocation`, `BrowserFavorites: Vec<PathBuf>`, `ThumbnailSize`, `ThumbnailCacheLimit`.
- **No PSD/XMP writes.** Metadata is read-only in v1 (editing, if ever, would be a separate feature, not Mini Bridge parity).
- **Undo:** opening/placing is a normal document transaction; browsing has no history state.
- **Cache:** thumbnail cache is a local on-disk/memory cache, keyed by `(path, mtime, size, thumb_size)`, never referenced by PSD.

## Edge cases

- **Missing/unreadable folder or file:** show an error row/inline message; do not stall the grid.
- **Huge folders:** virtualize the view; paginate/incremental scan; cap concurrent thumbnail jobs.
- **Huge images:** generate thumbnails from reduced decode, never load full raster into the UI thread.
- **Unsupported formats:** show a generic icon; opening fails with the normal "cannot open" error.
- **Symlinks and permission boundaries:** honour the file-access policy; do not follow symlinks out of allowed roots without the same checks as `AUTO-010`'s `File` sandbox.
- **Remote/portal mounts:** rely on `QFileSystemModel`/`GIO`-style portals; slow mounts must not block the UI.
- **Rapid folder switching:** cancel in-flight thumbnail jobs and drop stale results (guard by generation counter).
- **Unicode/long paths:** handle correctly on Linux; display with elision but preserve full path.
- **File changed on disk:** invalidate cache by `mtime`/size; refresh on window focus if requested.
- **No Bridge present:** irrelevant — the replacement has no such dependency; there is no "Waiting for..." state.
- **Dropping onto no document:** open a new document instead of failing.

## Parity acceptance criteria

1. Given a folder with mixed images, the Files panel shows thumbnails within the configured size, and scrolling a 10,000-file folder stays responsive (UI thread not blocked for more than a frame budget).
2. Given rapid switching between two folders, stale thumbnails from the abandoned folder are not shown.
3. Given a selected image, drag-and-drop onto the canvas opens it; with the Place modifier it becomes a Smart Object.
4. Given a file whose metadata is readable, the preview pane shows the documented EXIF/IPTC subset.
5. Given sort and filter changes, the grid updates without re-reading image pixels unnecessarily.
6. Given `File > Browse in Mini Bridge` is absent and `Window > Extensions > Mini Bridge` is absent or marked non-goal, the substitute `Window > Files` is present and functional.
7. Given a folder the process cannot read, the panel reports the error and remains usable.
8. Given the CS6 `Window > Extensions > Mini Bridge` menu entry, it is either not present or documented as a non-goal with a pointer to `Window > Files` (no half-working emulation).
9. Given 100 MB of thumbnails generated, the cache stays within the configured limit (LRU eviction verified).
10. Given a dropped file with no open document, a new document opens with that file's content.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — What's New: the Mini Bridge gallery is presented as offering easier access to images and documents via `Window > Extensions > Mini Bridge`; `File > Browse in Mini Bridge` and Mini Bridge links in the file-opening/workspace sections. Fetched via `curl` + `pdftotext`.
- `http://www.photoshopforphotographers.com/3101-1901/Help_guide/tp/Mini_Bridge.html` — Mini Bridge is a Photoshop extension panel over Bridge; `Window > Extensions > Mini Bridge`; simplified browsing (no keyword sorting, metadata editing, or web/print output). (Community/secondary.)
- `https://community.adobe.com/t5/photoshop-ecosystem-discussions/mini-bridge-panel-in-cs6/m-p/10118251` — CS6 Mini Bridge panel present but launch-Bridge button/icons missing because the Application bar was removed in CS6. (Community.)
- `https://community.adobe.com/questions-712/minibridge-error-waiting-for-bridge-cs6-1079673` — Adobe employee: Mini Bridge depends on **Bridge** and **SwitchBoard** (a network protocol connecting panel and Bridge); "Waiting for Bridge CS6" errors; Mini Bridge is Flash-based and was removed from Photoshop CC 2014 when Flash was removed; CEP/Flash extension model. (Community, with Adobe-employee responses.)
- `https://raw.githubusercontent.com/Adobe-CEP/CEP-Resources/master/README_ArchivedVersions.md` — CS6/CC 2013 support **CEP 4**, the Flash/ActionScript interface model; Flash/ActionScript support removed from CC2014+; extensions later use HTML5/JS CEP. (Adobe CEP documentation.)
- `https://github.com/Adobe-CEP/CEP-Resources` — corroborates that the Flash/ActionScript extension interface model is deprecated in Creative Cloud releases and that its support was removed from CC2014 onward.
- `https://en.wikipedia.org/wiki/Adobe_Bridge` — Mini Bridge as a small in-application file browser built on Bridge (community; seen in search results).
- Non-goal rationale cross-referenced with `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`) and `docs/00-overview/feasibility-and-non-goals.md`.

## Open questions

- **Was Mini Bridge "new" in CS6?** The CS6 Help says so, but Adobe CS5 material documents Mini Bridge in CS5. Resolution: check a CS5-era Adobe source; the CS6 Help wording appears to be recycled/marketing.
- **`File > Browse in Mini Bridge` presence in CS6.** The Help lists it; whether the entry existed in every CS6 build and locale needs confirmation from a real install or screenshots.
- **Exact CS6 degradation.** Community reports of the missing launch button vary; confirm whether some CS6 builds/icons worked and whether the panel was ever fully functional out of the box.
- **CEP version for CS6.** Adobe-CEP states CS6 supports CEP 4; whether Mini Bridge itself used CEP 4 (vs an earlier Flash panel) is not verified from a primary source.
- **Replacement scope.** Whether the native panel should also provide Bridge-style batch rename, metadata editing, and web/print output, or stay a browsable subset. Resolve with the `10-workflow-io/bridge-and-interop` spec.
- **Relationship to `10-workflow-io/bridge-and-interop` and `presets-manager`.** Overlap between the Files panel, the OS file dialog, and a future media browser must be resolved to avoid two browsers.
- **Camera import.** CS6's "Get Photos From Camera" was a Bridge feature; whether the Files panel should cover device import is deferred to non-goals.
