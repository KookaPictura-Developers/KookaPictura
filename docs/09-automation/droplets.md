# Droplets

- **Spec ID:** `AUTO-002`
- **Status:** `Draft`
- **Parity tier:** `Core` — droplets exist in CS6 Standard and Extended; the *executable-wrapper* form is platform-specific and is re-expressed on Linux (see below).
- **New in CS6:** `No` — `File > Automate > Create Droplet` and droplet playback are unchanged from CS5; droplet options follow the Batch options.
- **Depends on:** `AUTO-001` (actions), `AUTO-003` (batch-processing), `AUTO-010` (rust-scripting-replacement), `01-architecture/build-and-packaging`, `00-overview/feasibility-and-non-goals`.

## CS6 behavior

A **droplet** is a small application that applies an action to one or more images (or a folder of images) dragged onto its icon. Actions are the basis: the action must exist in the Actions panel before the droplet is created.

- `File > Automate > Create Droplet` opens a dialog with a **Save Droplet In** location, **Action Set** / **Action** selection, and the same **processing, saving, and file-naming options** as the Batch command (`AUTO-003`). Selecting the action in the Actions panel before opening the dialog preselects the menus.
- The resulting droplet is a self-contained program (on Windows a `.exe`, on macOS an application bundle, historically using the `.exe` extension to be cross-compatible). Double-clicking or dragging files/folders onto it starts Photoshop if it is not already running and processes them with the embedded action + options.
- **Cross-platform notes (documented):** a Windows-created droplet copied to macOS must first be dragged onto the Photoshop icon so Photoshop can update it; keeping the `.exe` extension when creating on macOS makes droplets usable on both; file/folder name references baked into the action (e.g. an `Open`/`Save As`/settings-loading step) do not cross operating systems — execution pauses and prompts for a filename.
- The droplet embeds the action **by reference to its content at creation**, not a live link to the panel; options captured at creation (override flags, destination, naming, errors) travel with the droplet.

### Linux behavior (proposed)

Photoshop never shipped for Linux, so there is no CS6 droplet binary to replicate. `00-overview/feasibility-and-non-goals` treats OS-specific desktop integration as a non-goal *parity* item; Kooka Pictura instead provides a **functional droplet equivalent**:

1. **`Kooka Pictura Droplet` (`.opd` / JSON spec)** — a text descriptor containing the embedded action set/action and the Batch options. Portable, diffable, and scriptable; this is the canonical droplet.
2. **Desktop launcher (`.desktop`)** — generated on request, with `Exec=kookapictura --droplet <path.opd> %F` and `MimeType`/`Type=Application`. File-manager drag-and-drop onto the launcher depends on the desktop environment (most XDG environments honour `%F`/`%U` arguments for `.desktop` files); where a DE does not, the CLI form is used.
3. **CLI** — `kookapictura --droplet <path.opd> <files-or-dirs...>` runs the same engine headlessly, which is also the sandbox/automation-friendly path. A thin `kookapictura-droplet` wrapper is proposed so the behaviour does not require the GUI.

The droplet engine is the Batch engine (`AUTO-003`): same source resolution, destination, naming, override flags, and error policy.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `File > Automate > Create Droplet…` | dialog | — | CS6 parity; action set/action, save location, Batch options. |
| Droplet icon | desktop/file-manager object | — | CS6: executable. Linux proposal: `.desktop` launcher and/or `.opd` file. |
| Drop target (file manager) | DnD | — | Drag files/folders onto the droplet; hidden in CS6, re-expressed as `%F`/`%U` on Linux. |
| `kookapictura --droplet <opd> <paths…>` | CLI | — | Proposed Linux equivalent; also usable from `AUTO-003` batch scripts. |
| `File > Automate > Batch…` | dialog | — | The dialog the droplet options are shared with (`AUTO-003`). |

## Parameters & ranges

The droplet dialog reuses the Batch/droplet options (`AUTO-003` documents the full table). Droplet-specific controls:

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Save Droplet In | path | user home | any writable path | CS6: `.exe`/app; Linux proposal: output dir for `.opd`/`.desktop`. |
| Action Set | enum | current | loaded action sets | Required. |
| Action | enum | current | actions in the set | Required. |
| Suppress File Open Options Dialogs | bool | on | on/off | Documented recommendation for camera-raw droplets. |
| All other options | — | — | see `AUTO-003` | Override Open/Save As, source, destination, naming, compatibility, errors. |

## Algorithms & pipeline

```text
create (GUI):
  read action set/action from Actions panel
  read Batch options from the dialog
  embed a self-contained ActionSet subset + options
  write .opd (canonical)   [Linux: optionally also emit .desktop]
  [CS6 analogue: compile a small executable that launches Photoshop]

run (drop / CLI):
  resolve inputs: each dropped file -> file; each dropped dir -> folder (recurse if Include All Subfolders)
  launch application if not running (CS6) / start batch engine (Linux headless or in-process)
  for each input:  open -> apply action steps -> destination -> save/close
  report/log errors per Error policy
```

- The droplet is **stateless at run time** except for its embedded action/options; it does not consult the live Actions panel.
- Because it is a self-contained descriptor, the Linux droplet can be run by a CI job, a file-manager action, or a `systemd` path unit, not only by the GUI.
- Path resolution and override semantics are delegated to `AUTO-003`; the droplet adds only the "inputs are the dropped paths" rule and launcher generation.

## Rust module mapping

- `crate::automation::droplet` — `DropletSpec { action_set: String, action: String, embedded: ActionSet, options: BatchOptions, name }`; `write_opd`/`read_opd`; `emit_desktop_launcher(path) -> DesktopEntry`.
- `crate::automation::batch` — the execution engine (`AUTO-003`); droplet directly constructs a `BatchJob` and calls it.
- `crate::automation::launcher` — `.desktop` entry generation and validation (`Exec`, `MimeType`, `Terminal=false`, exec quoting); never executed by the core, only written.
- `crate::cli` — argument parsing for `--droplet <opd> <paths…>` and `--create-droplet`; headless mode reuses the same `BatchJob`.
- `crate::action::atn` / `crate::action::model` — the embedded action set's serialisation (`AUTO-001`).

Boundary types: `BatchOptions` (shared with `AUTO-003`), `DropletSpec`, file-path lists.

## Qt6 component mapping

- `CreateDropletDialog` — mirrors the CS6 dialog: save location, set/action combos, embedded Batch options (shared `BatchOptionsWidget`), plus a Linux-only "Also create desktop launcher" checkbox.
- `BatchOptionsWidget` — reused verbatim by `BatchDialog` (`AUTO-003`), so the two dialogs cannot drift.
- Drop acceptance — the main window and any generated launcher accept `QDragEnterEvent`/`QDropEvent` (`QMimeData` URLs); dropped directories feed `BatchJob`.
- `QFileDialog` for the save location; `QDesktopServices::openUrl` only to *reveal* a created droplet, never to run it.

Widgets, not QML: the dialog is a modal utility over `QMainWindow` and shares its options widget with Batch. Desktop-launcher emission is plain file IO and has no Qt widget.

## Data-model impact

- **New persisted type (outside PSD):** `DropletSpec` (`.opd`, JSON) containing an embedded `ActionSet` and `BatchOptions`. It is not XMP and never written into a document.
- **Generated side artifacts:** `.desktop` launcher (optional), and the CS6 analogue would have been a binary — Kooka Pictura deliberately emits no opaque executable for droplets.
- **Undo:** droplets do not open documents interactively for the user, so their per-file mutations follow `AUTO-003` (batch) undo/memory policy, not the interactive history; no droplet-level history record is needed. Each processed file is opened, modified, saved, and closed.
- **Preferences:** the last-used droplet location and options persist with the Batch options.

## Edge cases

- **Action edited after droplet creation:** CS6 embeds content, so an edited action does not change an existing droplet. `DropletSpec` embeds a copy and documents this; re-create to update.
- **Action set/action missing at run time:** the embedded copy makes the droplet self-sufficient; a malformed/legacy `.opd` fails with a named error before processing any file.
- **Dropped paths with spaces/Unicode:** `.desktop` `Exec` quoting and CLI argument parsing must handle them; `%F` passes files as separate arguments, `%U` as URLs.
- **Directory with no images / unreadable files:** skip with a logged error; do not abort under `Log Errors to File`.
- **Destination = Save And Close onto originals:** overwrite semantics are inherited from `AUTO-003`; the droplet must not silently corrupt originals on partial failure (write-temp-then-rename where possible).
- **Headless (no display):** `--droplet` must run with no Qt GUI; only the core engine and file IO. Sandbox/permission constraints apply (`AUTO-010`).
- **Desktop-environment differences:** some file managers ignore `.desktop` `%F`/`%U`; document the CLI fallback rather than promising universal DnD.
- **Path baked into the action:** file/folder references in steps may prompt or fail; the CS6 cross-platform warning is preserved as a diagnostic at creation time.
- **Concurrent drops / same output file:** file locking and collision handling follow `AUTO-003`; a droplet run is serialised by default.
- **Symlink/`..` traversal in dropped folders:** respect `Include All Subfolders` but reject paths outside granted roots (`AUTO-010` sandbox).

## Parity acceptance criteria

1. Given an action set/action and a chosen destination, creating a droplet produces a runnable artifact (`kookapictura --droplet <opd> <file>`) that applies that action to the file and writes to the destination.
2. Given a folder dropped on the droplet with `Include All Subfolders` off, only top-level files are processed; with it on, subfolder files are processed too.
3. Given `Destination = Save And Close` and `Override Action "Save As" Commands`, processed files overwrite in place with original names; given `Destination = Folder`, they are written to the chosen folder under the naming scheme.
4. Given a droplet whose action contains an `Open` step for a specific file and `Override Action "Open" Commands` is on, the dropped files (not the baked-in file) are processed.
5. Given an action with no `Save As` step and destination `Save And Close`, files are not saved (CS6 behaviour) and the run reports it.
6. Given a droplet and an unreadable input, `Stop For Errors` halts with a message; `Log Errors to File` continues and writes the error file.
7. Given a generated `.desktop` launcher, its `Exec` line round-trips paths containing spaces and non-ASCII characters.
8. Given an edited source action after droplet creation, re-running the old droplet still applies the embedded (old) action until re-created.
9. Given the same `.opd` and inputs, a GUI run and a headless `--droplet` run produce byte-identical outputs.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — *Processing a batch of files*: *Create a droplet from an action*, *Tips for cross-platform droplets* (Windows→macOS re-init, `.exe` extension, filename references), *Process a file with a droplet*, *Batch and droplet processing options*. Fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-JavaScript-Ref.pdf` — `Application.batch(inputFiles, action, from [, options])` and `BatchOptions` property set used by the droplet dialog. Community mirror; fetched via `curl` + `pdftotext`.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — camera-raw droplet guidance, directing the user to `Suppress File Open Options Dialogs` in the Create Droplet dialog's Play area. Same PDF, scripting/Camera Raw chapter.
- Cross-reference `docs/09-automation/batch-processing.md` (`AUTO-003`) for the shared option set and execution engine, `docs/09-automation/actions.md` (`AUTO-001`) for the embedded action format, and `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`) for sandbox/CLI boundaries.
- Freedesktop `.desktop` entry semantics are assumed from the Desktop Entry Specification; **not fetched during this research** and therefore provisional (see Open questions).

## Open questions

- **Droplet file format.** `.opd` (JSON) is proposed; whether to also accept/produce an Adobe droplet binary (which embeds a compiled launcher and is platform-specific) is undecided and likely `Non-goal (Linux)`.
- **Desktop Entry support on target DEs.** GNOME/KDE/XFCE drag-and-drop of `.desktop` `%F`/`%U` was not verified against the Desktop Entry Specification during this research. Resolve by testing on the target distributions or by standardising on the CLI wrapper.
- **`%F` vs `%U`.** Which placeholder reliably yields filesystem paths across file managers needs empirical validation.
- **Indexing/launch behaviour.** CS6 starts Photoshop if it is not running; whether the Linux droplet should launch the GUI, run in-process, or run headless by default is a product decision (proposal: headless).
- **Error-file location and format.** CS6 writes an error log file; its path/format is not documented. Resolve from a real run or define a Kooka Pictura log format.
- **Permissions.** Dropped arbitrary paths vs the script sandbox roots (`AUTO-010`) must be reconciled; a droplet implicitly requests filesystem access to its inputs.
- **Cross-platform moving.** Whether to reproduce the CS6 "drag onto the app to re-initialise" step is moot on Linux; document the equivalent (regenerate) instead.
