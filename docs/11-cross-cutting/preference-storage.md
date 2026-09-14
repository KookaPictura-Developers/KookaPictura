# Preference Storage

- **Spec ID:** `XC-002`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — CS6 keeps the CS5 `.psp` preferences model but adds fields (Auto Save / Automatically Save Recovery Information, Save In Background, expanded Mercury/GPU settings, the 3D pane, new General/Interface options), and CS6 Help adds the "Preference filenames and locations" guidance. The storage *mechanism* (one version-specific binary `.psp` written on quit) is unchanged from CS5.
- **Depends on:** `02-ui-ux/preferences.md` (`UI-010`), `02-ui-ux/workspace-and-docks.md`, `02-ui-ux/keyboard-shortcuts.md` (`UI-011`), `01-architecture/rust-core-design.md`, `11-cross-cutting/localization.md` (`XC-001`), `11-cross-cutting/logging-and-telemetry.md` (`XC-003`), `11-cross-cutting/security-and-sandboxing.md` (`XC-005`).

> Rust crate names and the on-disk format below are **design proposals**. CS6's `.psp` is a proprietary binary; Kooka Pictura promises the *observable* persistence contract, not byte compatibility. Claims about the exact CS6 bytes or registry usage are marked *(secondary)*/`(unverified)`.

## CS6 behavior

### What CS6 stores and where

CS6 stores program settings in a single binary preferences file, **"Adobe Photoshop CS6 Prefs.psp"**, inside a version-specific Settings folder. Per the CS6 Help "Preference filenames and locations" page and community-verified CS6 paths (`UI-010`) *(secondary)*:

| Platform | Path |
|---|---|
| Windows (Vista/7/8/10) | `%APPDATA%\Adobe\Adobe Photoshop CS6\Adobe Photoshop CS6 Settings\Adobe Photoshop CS6 Prefs.psp` |
| Windows (XP) | `Documents and Settings\<user>\Application Data\Adobe\Adobe Photoshop CS6\…` |
| macOS | `~/Library/Preferences/Adobe Photoshop CS6 Settings/Adobe Photoshop CS6 Prefs.psp` |

A second file name, **"Adobe Photoshop x64 CS6 Prefs.psp"**, is reported by community threads for the 64-bit build *(secondary; naming inconsistent across reports)*. CS6 Help also documents a related "Settings" folder holding presets and scripts. Some preset libraries written from panels (brushes, swatches, styles, patterns, contours) are stored as separate libraries, not in the document.

### What the preferences contain

The CS6 Preferences dialog exposes panes that map one-to-one onto stored fields (`UI-010`): **General, Interface, File Handling, Performance, Cursors, Transparency & Gamut, Units & Rulers, Guides/Grid/Slices, Plug-ins, Type**, plus **3D** in Extended. Observable storage behavior:

- "Preference settings are saved each time you quit the application" (CS6 Help, via `UI-010`).
- Deleting or moving `… Prefs.psp` makes Photoshop recreate defaults on next launch.
- Holding `Alt+Ctrl+Shift` (Win) / `Option+Command+Shift` (Mac) at launch offers to delete the Settings/preferences file and reset defaults; CS6 Help notes this also resets custom shortcuts, workspaces, and color settings.
- **General → Reset All Warning Dialogs** re-enables suppressed "Don't Show Again" messages — i.e. suppressed-dialog state is itself persisted.
- A *corrupt* preferences file is the canonical cause of Photoshop acting strangely; the standard fix is to delete it (Photoshop Essentials, community).

### Registry and other stores *(secondary/unverified)*

On Windows, Photoshop historically also touches the registry for **licensing/activation, policy lockdown (`HKLM\SOFTWARE\Policies\Adobe\…\FeatureLockdown`), file-association and the plug-in cache**, and the Adobe Product Improvement Program can be disabled through a policy key. Whether CS6 mirrors any *preference* value in the registry (as opposed to activation/policy) is not established here and is left open. The preference store proper is the `.psp` file. CS6 additionally uses:

- **Workspaces** (panel layout) — separate saved workspace data, not the main prefs.
- **Keyboard shortcut sets** — user-defined sets (`UI-011`), reset by the same startup gesture.
- **Color settings** — a separate `.csf`-style settings file, reset by the same gesture.
- **Recent File List** — a bounded list persisted across sessions.
- **Per-document settings** (guides/grid overrides, slices, current tool options that travel with the doc) — belong to the document, not the preferences store.

### Versioning and migration

The Settings folder name is **version-specific** (`Adobe Photoshop CS6 Settings`). Adobe does not document any automatic import of a previous version's `.psp`; the common community guidance across upgrades is that preferences do not carry over reliably and users re-set or copy/rename them. There is no documented schema version, migration, or partial-merge behavior for `.psp`. Kooka Pictura therefore treats the CS6 `.psp` as a **migration source only if it is publicly documented later**, and otherwise starts from its own schema (independent-creation; see `00-overview/licensing-and-independent-creation.md`).

### Global vs. per-workspace vs. per-document

Three scopes are kept distinct in CS6:

1. **Application/global** preferences (the `.psp`): memory, GPU, cursors, units, file handling, type engine.
2. **Workspace** data: which panels are open/docked and where (named workspaces, `UI-010`/workspace docs).
3. **Per-document** overrides: guides, grid, slices, current selection/tool state that is saved with the file.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Edit > Preferences > <pane>` | Dialog | `Ctrl/Cmd+K` | Writes to the store on OK; most values apply live, several require restart. |
| `Edit > Preferences > General > Reset All Warning Dialogs` | Button | — | Persisted suppressed-dialog state. |
| `Edit > Preferences > File Handling` | Pane | — | Recovery/auto-save interval; Save In Background. |
| `Edit > Preferences > Performance` | Pane | — | Memory/scratch/GPU; large fraction of stored bytes. |
| `Alt+Ctrl+Shift` at launch | Startup gesture | — | Delete Settings file / reset. |
| `Edit > Keyboard Shortcuts` | Dialog | — | Stores shortcut sets (separate store). |
| `Window > Workspace` | Menu | — | Stores named workspaces (separate store). |
| Preferences file | Program data | — | `… Prefs.psp`; no in-app file UI. |

> Proposal (not CS6): a `Help → Open Configuration Folder` / `Reset Configuration` action and an import-from-CS6 assistant. CS6 has neither as a first-class UI.

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Store schema version | u32 | 1 | monotonic | Proposal; enables migration. |
| Write trigger | enum | on quit | on-quit / debounced / immediate | CS6 = on quit; live options apply immediately in-session. |
| Backups kept | int | 1 | 0–n | Proposal; one previous good copy. |
| Recent files | int | 20 *(secondary)* | 0–100 *(secondary)* | Bounded list. |
| Auto-save interval | minutes | 10 *(secondary)* | 5/10/15/30/60 *(secondary)* | Recovery store, separate from prefs. |
| Memory Usage | % RAM | 60–70 *(secondary)* | ~5–100 | Restart required. |
| History States | int | 20 *(secondary)* | 1–1000 | Restart-class. |
| Suppressed dialogs | set of IDs | empty | per-dialog IDs | "Reset All Warning Dialogs" clears. |
| Language | BCP-47 | system | catalogs | `XC-001`. |
| Concurrency policy | enum | last-writer-wins | last-writer / lock+merge | Proposal. |

## Algorithms & pipeline

### Proposed Linux storage (XDG)

Use the XDG Base Directory specification via the `directories` crate (`ProjectDirs`), so paths honor `$XDG_*` and defaults (`~/.config`, `~/.local/state`, `~/.local/share`, `~/.cache`):

| Purpose | Location (default) | Format |
|---|---|---|
| Global preferences | `$XDG_CONFIG_HOME/kooka-pictura/prefs.toml` (`~/.config/kooka-pictura/`) | TOML |
| Shortcut sets | `$XDG_CONFIG_HOME/kooka-pictura/shortcuts/*.toml` | TOML |
| Saved workspaces | `$XDG_STATE_HOME/kooka-pictura/workspaces/*.json` (`~/.local/state/…`) | JSON (opaque layout blobs) |
| Recent files / window state | `$XDG_STATE_HOME/kooka-pictura/state.json` | JSON |
| Preset libraries | `$XDG_DATA_HOME/kooka-pictura/presets/` (`~/.local/share/…`) | per-library format |
| Caches (thumbnails, decoded metadata) | `$XDG_CACHE_HOME/kooka-pictura/` | derived |
| Recovery/autosave | `$XDG_STATE_HOME/kooka-pictura/recovery/` | document snapshot |

Rationale: config is user-editable and portable → TOML (`toml` 1.x via `serde`); workspaces are machine layout blobs the user does not hand-edit → JSON. **Do not store application configuration via `QSettings` as the source of truth**; the Rust core owns the store, and the Qt shell reads it through the bridge. (If `QSettings` is used at all, it is a thin adapter over the same file, or limited to Qt-only window state; native `QSettings` on Linux writes `~/.config/<Org>/<App>.conf`, which would fragment the store.)

### Read/write pipeline

```text
startup:
  load defaults (compiled in)
  → layer system (/etc/xdg/kooka-pictura/prefs.toml, optional, read-only)
  → layer user ($XDG_CONFIG_HOME/kooka-pictura/prefs.toml)
  → parse; on parse error: quarantine bad file, load prefs.toml.bak, else defaults; warn
  → run schema migrations (vN → vN+1)
  → validate + clamp out-of-range values; drop unknown-but-preserve
  → publish PreferencesService snapshot

mutation:
  widget → PreferencesService.set(key, value) → validate → emit PrefsChanged
  → mark dirty; schedule debounced write (live options) or defer to quit

write (atomic):
  serialize complete document → temp file in same directory (0600)
  → fsync file → rename over prefs.toml → fsync directory
  → on failure: keep old file, surface a non-fatal warning

quit:
  flush pending writes; if store unchanged, do not rewrite
```

- **Atomicity:** temp-file + rename (the same guarantee Qt's `QSaveFile` gives: write to a temp file in the target directory and `commit()` renames it; a failure discards the temp file so the original is never half-written). Rust does this directly; if the Qt layer performs a write it must use `QSaveFile`, not `QFile`.
- **Corruption handling:** a parse/validation failure never crashes and never silently discards the user's settings. Rename the bad file to `prefs.toml.corrupt-<epoch>` (so it can be inspected/reported), fall back to the last-good `prefs.toml.bak`, then compiled defaults; log at WARN and show a one-time notice.
- **Forward/unknown keys:** unknown keys are retained on rewrite where possible (serde `#[serde(flatten)] extra: toml::Table`) so an older/newer build does not destroy settings it does not understand.
- **Schema versioning:** `schema_version` is the first key. Each migration is a pure `fn(Table) -> Result<Table>`; migrations run in order; a value higher than the build understands loads read-only with a warning rather than downgrading silently.
- **Concurrency:** two running instances must not clobber each other. Proposal: an advisory lock file in the config dir plus a read-modify-write merge on save; if locking is unavailable, last-writer-wins with a debounce and a logged warning. `QSettings` on INI uses advisory locking and smart merging; if `QSettings` is the writer we inherit that, but the core writes its own file.
- **Scopes:** global prefs in `prefs.toml`; workspace layout and recent-file state in `state.json`/`workspaces/` so resetting preferences does not destroy the user's workspace, and vice versa. Per-document overrides never enter this store.
- **CS6 `.psp` import:** off by default and legal-gated. If a independent-creation reader is ever written, it is an explicit `File → Import → CS6 Preferences` action, and its failure must never affect the native store (sandbox the parse; `XC-005`).

### Restart-class vs. live settings

Reuse `UI-010`'s semantics: most fields apply immediately; Memory Usage, History States/Cache, UI Font Size, and some GPU options require a restart. The schema records a `restart_required` flag per field so the UI can prompt and the loader can stage the change.

## Rust module mapping

- `pictura-prefs::Prefs` — one `#[derive(Serialize, Deserialize)]` struct per pane (`GeneralPrefs`, `InterfacePrefs`, `FileHandlingPrefs`, `PerformancePrefs`, `CursorPrefs`, `TransparencyPrefs`, `UnitsPrefs`, `GuidesGridSlicesPrefs`, `PluginsPrefs`, `TypePrefs`, `ThreeDPrefs`) plus `extra: toml::Table` for unknown keys.
- `pictura-prefs::PrefsSchema` — field metadata (key, type, default, range, pane, restart-required). Single source of truth for the Qt dialog and validation.
- `pictura-prefs::Store` — load/save; atomic temp+rename; `.bak` rotation; parse-failure quarantine; `fsync`.
- `pictura-prefs::migrate` — ordered `schema_version` migrations with `MigrationError`.
- `pictura-prefs::paths` — `config_dir()`, `state_dir()`, `data_dir()`, `cache_dir()` via `directories::ProjectDirs`; `prefs_path()`, `state_path()`.
- `pictura-prefs::PreferencesService` — `get::<T>()`, `set(...)`, typed change subscriptions; emits `PrefsChanged { key, old, new }`.
- `pictura-prefs::Scope` — `Global | Workspace | Document`; routes reads to the correct store.
- `pictura-prefs::cs6_import` — optional, feature-gated, off by default.
- Types crossing the Rust↔Qt boundary: the pane structs, `PrefsChanged`, `PrefsFieldMeta`, `Scope`. Plain data; no Qt types in Rust.

## Qt6 component mapping

- `PreferencesDialog` / `PreferencesPage` (`QDialog`/`QWidget`) — pane list + `QStackedWidget`, built from `PrefsSchema` (`UI-010`).
- `PrefsController` (`QObject`, cxx-qt) — Q_PROPERTYs and slots over `PreferencesService`; `changed(key)` signal.
- `QStandardPaths` — verify the XDG locations agree with `pictura-prefs::paths`; use `AppConfigLocation`/`AppStateLocation` only as a cross-check, not a second writer.
- `QSaveFile` — if Qt performs any store write, use it for atomic commit; never `QFile` for the store.
- `QSettings` — **not** the source of truth; optional read-only adapter for Qt-only window geometry, or omitted.
- `ConfigurationResetDialog` — the `Alt+Ctrl+Shift` startup gesture and a Preferences button; backs up rather than deletes by default (proposal).

## Data-model impact

- **Not document data.** Preferences never serialize into PSD/XMP and never enter the History panel (`UI-010`).
- **Application store** carries its own `schema_version`; migration is independent of document versions.
- **Window/workspace state** lives in the state layer, so "delete preferences" does not necessarily destroy workspaces unless the user asks to reset those too (CS6's reset gesture resets both, so the reset dialog must offer the same combined reset).
- **Suppressed dialog IDs** are stored as stable string IDs, not message text, so `XC-001` translations do not break suppression.
- **Recent files** store canonical absolute paths plus a display label; paths are validated on load (missing/permission-denied entries are dropped or greyed).
- **Recovery data** (auto-save snapshots) belongs to the recovery spec, not this store, but its location and retention are configured here.
- **Undo:** preference edits are not document-undoable; the Preferences dialog's Cancel must roll back uncommitted widget state without touching the store.

## Edge cases

- **Missing store:** seed defaults and write on first quit; do not crash.
- **Corrupt store:** quarantine + last-good backup + defaults, with a notice; never silently lose settings.
- **Read-only / unwritable config dir:** run in-memory with a warning; do not lose the session; retry on next launch.
- **Disk full during atomic write:** temp write or rename fails; original remains intact; surface a non-fatal warning.
- **Concurrent instances:** advisory lock + merge, or last-writer-wins with a warning; no torn files (atomicity protects this).
- **Interrupted write (crash/power loss):** rename is atomic, so the store is either old or new; a leftover temp file is cleaned on next start.
- **Forward-incompatible version:** load read-only and warn rather than downgrade.
- **Unknown keys:** preserve on rewrite.
- **Locale:** numeric fields parse/store canonically (`.` decimal), display via `QLocale` (`XC-001`); never let a locale separator enter the stored TOML.
- **Flatpak:** `$XDG_CONFIG_HOME` and friends are redirected under `~/.var/app/<id>/…`; the app must not hardcode `~/.config` (`XC-005`).
- **Unicode paths / filenames:** store UTF-8; never use `Str255`-style legacy encodings.
- **Migration from CS6:** `.psp` is proprietary and possibly encumbered; default path is a fresh profile, not an import.
- **Reset gesture while another instance runs / file locked:** refuse and warn.
- **Huge recent list / many workspaces:** cap and prune; never grow without bound.

## Parity acceptance criteria

1. Given a fresh profile, launching creates the config directory with `0700`-compatible semantics and a valid `prefs.toml`; all panes report CS6 defaults.
2. Given a changed setting, quitting and relaunching restores it; changing a restart-class setting shows the restart prompt and applies after relaunch.
3. Given the store is deleted, relaunching recreates defaults; given `Alt+Ctrl+Shift` is held at launch and confirmed, preferences, shortcuts, workspaces, and color settings reset together.
4. Given a truncated/corrupt `prefs.toml`, the app starts with defaults, preserves the corrupt file as `prefs.toml.corrupt-<epoch>`, restores the last-good backup if present, and warns once.
5. Given the destination directory is read-only, the app runs with in-memory settings, warns, and does not crash on quit.
6. Given two instances edit preferences, the final file is valid (no torn parse) and neither instance silently discards the other's parseable keys.
7. Given a settings file containing unknown keys from a newer build, rewriting the file preserves those keys.
8. Given a `decimal-comma` locale, numeric preference fields round-trip through the UI without corrupting stored values.
9. Given `Reset All Warning Dialogs`, previously suppressed dialogs reappear after restart.
10. Given a Flatpak run, the store lands under the app's sandboxed config location, not the host `~/.config`.

## Sources

Fetched for this document:

- `https://specifications.freedesktop.org/basedir-spec/latest/` — XDG Base Directory Specification 0.8: `$XDG_CONFIG_HOME` (default `~/.config`), `$XDG_STATE_HOME` (`~/.local/state`), `$XDG_DATA_HOME`, `$XDG_CACHE_HOME`, `$XDG_CONFIG_DIRS` (`/etc/xdg`), absolute-path rule, `0700` creation, write-failure guidance.
- `https://docs.rs/directories/latest/directories/` — `directories` 6.0 `BaseDirs`/`ProjectDirs`/`UserDirs`; XDG on Linux, Known Folders on Windows, Standard Directories on macOS.
- `https://docs.rs/toml/latest/toml/` — `toml` 1.1 serde-compatible encoder/decoder; `Table`/`Value`, `from_str`/`to_string_pretty`, `serde` support, `Spanned`.
- `https://doc.qt.io/qt-6/qsavefile.html` — `QSaveFile` atomic write semantics (temp file + `commit()` rename; original preserved on failure; `setDirectWriteFallback` breaks atomicity — keep the default for config).
- `https://doc.qt.io/qt-6/qsettings.html` — `QSettings` formats/scopes; Unix `NativeFormat`/`IniFormat` paths (`$HOME/.config/<Org>/<App>.conf`); INI-advisory locking + smart merge; `FormatError`/`AccessError` statuses; registry/plist behavior on other platforms.
- `https://www.photoshopessentials.com/basics/reset-photoshop-preferences/` — *(secondary)* Photoshop preferences file is rewritten on every quit and is the usual corruption cause; delete-to-reset and the `Shift+Ctrl+Alt` / `Shift+Command+Option` relaunch gesture; CS6 has no "Reset Preferences On Quit" (that is CC 2015+).
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — **primary CS6 reference; not re-fetched in this pass**. `.psp` storage, saved-on-quit behavior, reset gesture, Reset All Warning Dialogs, and pane set are cited via `docs/02-ui-ux/preferences.md`.

Found via search, **not fetched** (leads):

- `https://helpx.adobe.com/photoshop/kb/preference-file-names-locations-photoshop.html` — returned HTTP 403. Would establish the authoritative per-version preference-file names/locations.
- `https://community.adobe.com/t5/photoshop-ecosystem-discussions/can-t-open-new-or-existing-files/td-p/10153134` — Adobe Community: CS6 `.psp` file names (`Adobe Photoshop CS6 Prefs.psp`, `Adobe Photoshop x64 CS6 Prefs.psp`), used to corroborate the second file name.
- `https://community.adobe.com/questions-712/how-to-reset-preferences-if-i-have-the-legacy-version-1140349` and `https://community.adobe.com/t5/photoshop-ecosystem-discussions/how-do-i-save-preference-changes-in-ps6/m-p/8864474` — Windows/macOS CS6 paths (search-result evidence only; already recorded in `UI-010`).

Internal cross-references (not sources): `docs/02-ui-ux/preferences.md` (`UI-010`), `docs/00-overview/licensing-and-independent-creation.md`.

## Open questions

- **Exact CS6 preference-file name(s) and whether 32-/64-bit builds use different files.** Community reports disagree on `… CS6 Prefs.psp` vs `… x64 CS6 Prefs.psp`. *Resolve:* the Adobe "Preference file names, locations" page (403) or a CS6 install.
- **Does CS6 store any *preference* value in the Windows registry, or only activation/policy?** *Resolve:* registry diff on a clean CS6 profile.
- **`.psp` format.** Proprietary/unknown. *Resolve:* independent-creation legal review; default is no import.
- **Whether CS6 migrates any settings between versions.** Not documented. *Resolve:* test CS5→CS6 on the same machine.
- **Whether workspaces/shortcuts/color settings share the `.psp` or live in separate files.** CS6 Help says the reset gesture resets all four but does not say where each lives. *Resolve:* inspect the CS6 Settings folder layout.
- **TOML vs JSON for the native store.** This spec proposes TOML for config and JSON for state; a single format (TOML everywhere) is a reasonable alternative. *Resolve:* prototype and settle the `UI-010` open question.
- **Concurrency policy.** Advisory lock vs. last-writer-wins for the common "two windows edit preferences" case. *Resolve:* decide with `01-architecture/threading-and-concurrency.md`.
- **Quarantine retention and privacy.** Keeping `.corrupt-*` files helps support but may retain user data; decide retention/redaction. *Resolve:* security review (`XC-005`).
