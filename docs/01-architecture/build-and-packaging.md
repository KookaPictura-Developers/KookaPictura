# Build and Packaging

- **Spec ID:** `ARCH-014`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — distribution mechanism is implementation-defined; nothing here mirrors a CS6 feature.
- **Depends on:** `ARCH-001`, `ARCH-002`, `ARCH-003`. Consumed by the packaging/CI parts of `11-cross-cutting/`.

This document specifies how Kooka Pictura is built and distributed on Linux:
the CMake↔Cargo integration model, the four packaging targets, display-server
support (X11 + Wayland), GPU driver assumptions, and license/compliance
obligations. Nothing here is implemented; this is a design specification.
Tooling names are **design proposals** unless they are documented Qt/Flatpak
mechanisms.

## CS6 behavior

Not applicable: CS6 shipped as installer/updater bundles for Windows and macOS,
not as Linux packages. The transferable CS6-side constraints are the minimum
hardware/OS floors that Kooka Pictura's packages must document to users:

- Photoshop CS6 64-bit; a 64-bit OS is required (macOS CS6 is 64-bit only).
- Minimum 1024×768 display (1280×800 recommended), 16-bit color.
- GPU floors: 256 MB VRAM minimum, 512 MB for 3D; OpenGL 2.0 + Shader Model 3.0
  for OpenGL acceleration; OpenCL 1.1 for the Blur Gallery.
- ~2 GB free disk for the application plus scratch space; Adobe recommended a
  dedicated scratch volume separate from the OS swap file.

These become the **documented runtime requirements** for each Linux package.

## UI surface

Packaging surfaces only through distribution metadata and first-run behavior.

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `.desktop` entry | Desktop file | — | Name, icon, categories, `StartupWMClass`, Wayland/X11 hints. |
| AppStream metadata | XML | — | Descriptions, screenshots, release notes for software centers. |
| MIME definitions | Shared MIME info | — | PSD/PSB/TIFF/etc. so the app is offered as an opener. |
| First-run notice | Dialog | — | GPU/VRAM report and OpenGL/OpenCL availability. |
| `--version` / `About` | CLI/dialog | — | Build, Qt, GPU backend, and codec versions for support. |

## Parameters & ranges

Build-time switches and runtime environment knobs the packaging must expose.

| Control | Type | Default | Options | Notes |
|---|---|---|---|---|
| Graphics backend | Runtime | `auto` | `vulkan`, `opengl`, `null` | Maps to `wgpu`/QRhi backend selection. |
| Qt QPA platform | Env / CLI | system | `wayland`, `xcb` | `-platform` / `QT_QPA_PLATFORM`. |
| Rendering hardware | Env | on | on/off | Software fallback for unsupported GPUs. |
| Sandbox socket | Flatpak manifest | `wayland` + `fallback-x11` | — | From Flatpak's documented Qt example. |
| DRM device access | Flatpak manifest | `--device=dri` | — | Required for GPU rendering in the sandbox. |
| IPC sharing | Flatpak manifest | `--share=ipc` | — | X11/shm performance. |
| Scratch directory | Runtime pref | XDG data dir | any writable path | Must survive Flatpak/Snap sandboxing. |
| Package format | Build target | `deb` | `deb`, `rpm`, `flatpak`, `appimage`, `snap` | CI matrix, not a user control. |

## Algorithms & pipeline

### Build integration (CMake ↔ Cargo)

Two supported shapes, matching documented tooling:

1. **CMake-first (proposed default).** CMake is the top-level build. Cargo builds
   the Rust workspace as a `staticlib`; CMake links it and builds the C++/QML
   shell. The bridge is **cxx-qt** with **Corrosion** (`cxx_qt_import_crate`),
   which is the documented cxx-qt CMake path. C++ `main` starts
   `QGuiApplication`/`QQmlApplicationEngine`; Rust QObjects are included from the
   generated `<crate>/src/<file>.cxxqt.h` headers.
2. **Cargo-first.** Cargo builds the whole app (including a Qt-enabled wrapper)
   and `pictura-app` is the entry point. Simpler for a pure-Rust shell; less
   natural if most UI is C++/QML.

```text
CMake (top)                         Cargo workspace (crates/)
  ├─ find_package(Qt6)                ├─ pictura-core
  ├─ cxx_qt_import_crate ────────────►├─ pictura-*  → libpictura_qt.a (staticlib)
  ├─ qt_add_executable(pictura)       └─ target/ (shared)
  └─ qt_generate_deploy_app_script (Qt ≥6.5) → self-contained dir → CPack
```

Qt documents a CMake deployment API (Qt 6.5+) that writes `qt.conf`, uses
`GET_RUNTIME_DEPENDENCIES` to discover Qt libs/plugins, and installs them next to
the executable; the same doc shows `cpack -G DEB` for a `.deb`. There is no
`linuxdeployqt` in Qt 6.5+; deployment is CMake-native. `linuxdeployqt-qt6` exists
as a third-party Qt6 fork for AppImage-style bundling but is not official.

### Packaging targets

| Format | Mechanism (documented) | Strengths | Constraints |
|---|---|---|---|
| **Flatpak** | `org.kde.Platform` + `org.kde.Sdk` runtime; `buildsystem: cmake-ninja`; `finish-args: --share=ipc --socket=fallback-x11 --socket=wayland --device=dri` | Distro-agnostic, sandboxed, bundled Qt/KF | Sandbox limits scratch/file access; needs portals; Mesa/driver comes from host. |
| **AppImage** | Bundle Qt libs + QPA plugins (`platforms/libqxcb.so`, `wayland`) into an AppDir; run via launcher script setting `LD_LIBRARY_PATH` or RPATH | Runs anywhere, no install | Bundled Qt/driver assumptions; Wayland support is fragile (needs `qt6-wayland` platform plugin and `QT_QPA_PLATFORM`). |
| **Snap** | `snapcraft.yaml` with `qt6` plugin/base, `plugs` for `opengl`, `wayland`, `x11`, `home`, `removable-media` | Auto-updates, confinement | Strict confinement frequently fights GPU/scratch paths; classic confinement often needed. |
| **Native `.deb` / `.rpm`** | CPack (`-G DEB`, `-G RPM`) or distro packaging; `Depends: qt6-*`, `lcms2`, driver libs | Smallest, best desktop integration | Must match distro Qt versions; too many distros to cover directly. |

Recommended release set (proposal): **Flatpak** as the primary sandboxed build,
**native `.deb`/`.rpm`** for mainstream distros, and **AppImage** as the portable
fallback. Snap optional; its confinement burden may not be justified.

### Display server support (X11 + Wayland)

Qt exposes both through QPA plugins: `-platform xcb` (X11) and
`-platform wayland`. The Flatpak example requests both sockets
(`fallback-x11` + `wayland`), which matches a same-binary deployment.

- Prefer the Wayland platform plugin when available (`EXT_platform_wayland`
  enables buffer sharing); otherwise run under XWayland.
- Wayland is stricter than X11 for legacy functionality (the Qt doc explicitly
  warns about this), so global-shortcut-style and screen-capture features need
  portal-based paths, not X11 hacks.
- AppImage is the weakest for Wayland: verify the bundled `wayland` platform
  plugin exists and document `QT_QPA_PLATFORM=wayland` as the opt-in.
- Mixed-DPI multi-monitor is a known Wayland pain point; see Open questions.

### GPU driver assumptions

- Rendering is backend-agnostic at the API level: `wgpu` / QRhi select
  **Vulkan**, **OpenGL**, or a **null/software** backend at runtime. Flatpak's
  `--device=dri` grants the DRI render nodes needed by Mesa/Vulkan.
- Minimum target: Vulkan 1.0 (QRhi documents Vulkan 1.0+ with optional 1.1
  features; `wgpu` generally targets Vulkan 1.x on Linux). OpenGL is the fallback
  for older drivers; a software (`lavapipe`/`llvmpipe`) path must exist for VMs
  and CI even if slow.
- Do not assume a vendor: Intel/AMD/NVIDIA and open/closed drivers all must work
  through the same libdrm/Mesa/Vulkan loader interfaces. NVIDIA proprietary
  drivers need their Vulkan ICD present.
- Ship no GPU driver; depend on the host stack. Detect and report missing Vulkan
  ICDs at first run instead of crashing.
- Preserve the `ARCH-003` CPU fallback: no feature may become *impossible* only
  because the GPU is absent, except where the format/output genuinely requires
  GPU semantics (none currently identified).

### Bundled codecs and compliance

Codecs are linked as Rust crates, not vendored C. Licenses verified from
crates.io at time of writing:

| Crate | License | Use |
|---|---|---|
| `image` | MIT OR Apache-2.0 | PNG/JPEG/TIFF/… dispatch |
| `zune-jpeg` | MIT OR Apache-2.0 OR Zlib | JPEG decode |
| `qoi` | MIT/Apache-2.0 | QOI decode/encode |
| `lcms2` | MIT (wraps Little CMS) | ICC color transforms |
| `kamadak-exif` | BSD-2-Clause | EXIF metadata |
| `wgpu` | MIT OR Apache-2.0 | GPU rendering |
| `rayon` | MIT OR Apache-2.0 | CPU parallelism |
| `serde` | MIT OR Apache-2.0 | Serialization |
| `half` | MIT OR Apache-2.0 | f16 types |
| `thiserror` / `anyhow` | MIT OR Apache-2.0 | Error model |
| `cxx-qt` | MIT OR Apache-2.0 | Rust↔Qt bridge |
| `qmetaobject` | MIT | Alternative Rust↔QML |

Compliance rules for the release team:

- Preserve all copyright/permission notices; generate a `THIRD-PARTY-LICENSES`
  file and ship it in every package.
- Keep permissive dependencies dynamically or statically linked as their licenses
  allow (MIT/Apache/BSD are compatible with a distributed binary that includes
  notices).
- **Qt is the hard case.** Qt is dual-licensed; the open-source path is LGPLv3
  (some modules GPLv3-only). LGPLv3 requires that users can replace/re-link the
  Qt library and that corresponding source be delivered. Static linking of Qt (an
  option for AppImage) pulls the application toward LGPL/GPL obligations. The
  lazy, safe choice if the project is closed-source is **dynamic linking against
  system/runtime Qt** and shipping LGPL notices; if the project itself is
  GPL-compatible, static linking is acceptable. Record the decision in
  `licensing-and-independent-creation.md` (planned) after legal review.
- Do not bundle Adobe code, profiles, or assets. ICC profiles and example files
  must have their own redistribution rights.
- Patent-sensitive codecs (e.g. HEVC if ever added) require a separate review;
  none of the core formats above inherently require a patent license from a
  permissive implementation.

## Rust module mapping

| Module (proposed) | Build/packaging responsibility |
|---|---|
| `pictura-app` | Binary entry point; `--version`, backend detection report. |
| `pictura-qt` | `staticlib` crate imported by CMake; generates QML module. |
| `pictura-core::platform` | XDG paths, scratch-dir resolution under sandboxing. |
| `pictura-render::backend` | Runtime backend selection (`vulkan`/`opengl`/`null`). |
| `xtask` (proposed) | Build/packaging driver: invokes CMake, CPack, Flatpak builder. |

## Qt6 component mapping

| Component | Packaging responsibility |
|---|---|
| `qt_generate_deploy_app_script` (CMake, Qt ≥6.5) | Produce the self-contained install dir and `qt.conf`. |
| `CPack` (`-G DEB`, `-G RPM`) | Native packages from the deployed tree. |
| `QCommandLineParser` | `--version`, `--platform`, backend override. |
| `QSettings` / XDG portals | Persist prefs and scratch paths under confinement. |
| `QStandardPaths` | Correct config/data/cache locations across X11/Wayland/Flatpak. |

## Data-model impact

None. Packaging does not change document state. It does constrain where state may
live: preferences, scratch, and autosave must route through `QStandardPaths`/XDG
so they remain writable inside Flatpak/Snap sandboxes.

## Edge cases

- **Flatpak scratch:** a sandboxed app cannot use an arbitrary host path; scratch
  must live under the app's granted directories or be exposed via portal.
- **AppImage + Wayland:** missing `wayland` platform plugin or `QT_QPA_PLATFORM`
  causes XWayland fallback or launch failure.
- **Snap confinement:** GPU and removable-media access often require manual
  interfaces; classic confinement is a common escape hatch and needs review.
- **No Vulkan ICD / broken driver:** first-run detection must fall back to
  OpenGL, then software, and report clearly.
- **Distro Qt version skew:** native packages linked against an older Qt must
  state `Depends` precisely; the Flatpak runtime avoids this by construction.
- **HiDPI fractional scaling:** Wayland fractional scaling and X11 scaling behave
  differently; packaging must not hard-code a scale factor.
- **Static Qt (AppImage) license:** static linking changes obligations; see the
  compliance rules.
- **Non-x86_64 (aarch64):** Vulkan and codec availability differ; the CI matrix
  must state supported architectures.

## Parity acceptance criteria

1. Given a clean machine with no Qt installed, the Flatpak build launches, opens
   a PSD, and uses the host GPU via `--device=dri`.
2. Given a distro with Qt6 installed, the native `.deb` (or `.rpm`) installs,
   registers MIME handlers for PSD/PSB, and launches from the desktop menu.
3. Given an AppImage on a Wayland session, the app launches with
   `QT_QPA_PLATFORM=wayland` and renders; with `xcb` it runs under XWayland.
4. Given a machine without Vulkan, the app selects OpenGL; with neither, it
   selects the software path and still opens a document.
5. Given any package, `--version` reports build, Qt, GPU backend, and codec
   versions, and a `THIRD-PARTY-LICENSES` file is installed.
6. Given an LGPLv3 dynamic-linking configuration, Qt libraries remain replaceable
   by the user (no static-only Qt in the shipped artifact unless the project is
   GPL-compatible).

## Sources

- `https://doc.qt.io/qt-6/linux-deployment.html` — shared vs. static deployment, Qt plugins (`platforms/libqxcb.so`), `qt.conf`, CMake deployment API (Qt ≥6.5), CPack `-G DEB`.
- `https://doc.qt.io/qt-6/packaging-recommendations.html` — Qt versioned-tool packaging layout for distributions.
- `https://doc.qt.io/qt-6/wayland-and-qt.html` — Wayland/X11 QPA selection, `EXT_platform_wayland`, legacy-functionality caveats, multi-process trade-offs.
- `https://doc.qt.io/qt-6/qrhi.html` — QRhi backend list (Vulkan 1.0+, OpenGL, null), software-renderer flag.
- `https://docs.flatpak.org/en/latest/qt.html` — `org.kde.Platform`/`org.kde.Sdk`, `cmake-ninja`, `finish-args` (`--share=ipc`, `--socket=fallback-x11`, `--socket=wayland`, `--device=dri`).
- `https://kdab.github.io/cxx-qt/book/getting-started/5-cmake-integration.html` — cxx-qt `staticlib`, `build.rs` via `cxx-qt-build`, Corrosion `cxx_qt_import_crate`, generated-header include path.
- `https://kdab.github.io/cxx-qt/book/` — cxx-qt bridge model and platform support.
- `https://www.qt.io/licensing/open-source-lgpl-obligations` — Qt dual licensing, LGPLv3 obligations, static vs. dynamic linking, GPL-only modules.
- `https://web.archive.org/web/20140204041700/http://blogs.adobe.com/crawlspace/2012/10/how-to-tune-photoshop-cs6-for-peak-performance.html` — CS6 runtime floors (64-bit, display, VRAM), scratch-volume guidance.
- `https://web.archive.org/web/20140401152634/http://helpx.adobe.com/photoshop/kb/photoshop-cs6-gpu-faq.html` — 256/512 MB VRAM, OpenGL 2.0 + SM3.0, OpenCL 1.1.
- crates.io API (`https://crates.io/api/v1/crates/<name>`) — verified crate licenses listed in the compliance table.

## Open questions

- **Qt licensing choice** (LGPLv3 dynamic vs. GPL static/commercial) is unresolved and blocks the AppImage/static decision. *Resolves with:* legal review recorded in `licensing-and-independent-creation.md`.
- **Primary package format** is a product decision. *Resolves with:* a distribution-strategy decision; Flatpak is the working default.
- **Snap viability** under GPU/scratch confinement is unknown. *Resolves with:* a confinement prototype.
- **AppImage Wayland reliability** with a bundled Qt 6 is unverified. *Resolves with:* a Wayland CI smoke test on multiple desktops (GNOME/KDE/Hyprland).
- **Minimum Vulkan/OpenGL versions** to support are not fixed. *Resolves with:* a hardware survey and a documented support matrix.
- **Architecture coverage** (x86_64 only, or aarch64 too) is undecided. *Resolves with:* CI capacity and a target-user decision.
- **Whether `linuxdeployqt-qt6` is needed at all** given the Qt ≥6.5 CMake deploy API. *Resolves with:* an AppImage spike using only CMake deployment.
