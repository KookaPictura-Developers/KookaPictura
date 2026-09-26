# Batch Processing and the Automate Menu

- **Spec ID:** `AUTO-003`
- **Status:** `Draft`
- **Parity tier:** `Core` for `Batch`, `Image Processor`, `Conditional Mode Change`, `Crop and Straighten Photos`, `Photomerge`, `Merge To HDR Pro`, `PDF Presentation`, and `Contact Sheet II`; `Picture Package` and `Web Photo Gallery` ship as **optional plug-ins** in CS6 and are proposed as optional modules. `Photomerge` 360°/3D wrapping is `Extended-only`; `Merge To HDR Pro` tone-mapping is `Core`.
- **New in CS6:** `Changed` — **Contact Sheet II** and **PDF Presentation** returned as Automate options, updated and 64-bit compatible. `Picture Package`, `Contact Sheet`, and `Web Photo Gallery` moved to optional downloads / Adobe Bridge. Batch itself is CS5-equivalent.
- **Depends on:** `AUTO-001` (actions), `AUTO-002` (droplets), `AUTO-004` (script-events-and-jsx), `AUTO-010` (rust-scripting-replacement), `04-image-ops/32-bit-hdr`, `10-workflow-io/file-info-and-metadata`, `01-architecture/threading-and-concurrency`.

## CS6 behavior

### File > Automate > Batch

`Batch` runs an action over a set of files. The action is chosen by **Set** and **Action**; the menus list only actions currently loaded in the Actions panel.

**Source:**

- `Folder` — a chosen folder (with optional `Include All Subfolders`).
- `Import` — from a digital camera/scanner or a PDF, via an acquire plug-in that supports actions. A third-party importer not written for multi-document import may not work during batch.
- `Opened Files` — all open documents.
- `Bridge` — selected Bridge files, or the current Bridge folder if none are selected.

**Destination:**

- `None` — leave files open without saving (unless the action itself saves).
- `Save And Close` — save in place, overwriting originals.
- `Folder` — save to a chosen folder, with a **File Naming** scheme.

**Override flags** adapt an action to the batch:

- `Override Action "Open" Commands` — the batch-selected files are opened instead of the file baked into the action's `Open` step. (Without it, if the action contains an `Open`, only the baked-in file is repeatedly opened and the source files are never processed.)
- `Override Action "Save As" Commands` — processed files are saved to the batch destination/name instead of the folder/name in the action's `Save As` step. The action **must contain a `Save As` step**; options inside it (e.g. JPEG compression, TIFF options) are retained while filename/folder are overridden. If not selected and the action names a file, the batch overwrites that same file for every input.

**Naming:** `File Naming` combines fields (document name, serial number, serial letter, date, etc.) into the output name; at least one unique field is required to avoid overwrites. `Starting Serial Number` sets the first serial; serial letters begin at `A`. `Compatibility` makes names safe for Windows/macOS/UNIX.

**Errors:** `Stop For Errors` suspends on the first error; `Log Errors to File` records each error and continues, showing a message at the end.

**Other documented behaviour:** to batch multiple actions, wrap them in a new action that plays the others; to batch multiple folders, put aliases to them in one folder and enable `Include All Subfolders`. For speed, reduce saved history states and disable `Automatically Create First Snapshot`. Nested-folder processing into different formats uses `Destination = Save And Close` + `Override Action "Save As" Commands`, relying on the action's recorded options.

The scripting equivalent `Application.batch(inputFiles, action, from [, options])` takes a `BatchOptions` object exposing the same controls: `destination`, `destinationFolder`, `errorFile`, `fileNaming`, `overrideOpen`, `overrideSave`, `startingSerial`, `suppressOpen`, `suppressProfile`, `macintoshCompatible`, `unixCompatible`, `windowsCompatible`.

### File > Scripts > Image Processor

`Image Processor` converts/processes multiple files **without an action**: convert to JPEG, PSD, and/or TIFF simultaneously; apply the same Camera Raw settings; resize to fit; convert to sRGB and embed the profile; include copyright metadata; and optionally run an action. It works on PSD, JPEG, and Camera Raw files.

Controls: process open files or a folder; `Open First Image To Apply Settings`; save location; per-format options (JPEG quality `0`–`12`, `Resize To Fit`, `Convert Profile To sRGB`, PSD `Maximize Compatibility`, TIFF `LZW Compression`); `Run Action` (set/action), `Copyright Info`, `Include ICC Profile`; `Save`/`Load` settings.

### Other Automate commands

| Command | Menu path | Behaviour |
|---|---|---|
| Conditional Mode Change | `File > Automate > Conditional Mode Change` | Records a source-mode set + target mode as an action step (`AUTO-001`). |
| Crop and Straighten Photos | `File > Automate > Crop And Straighten Photos` | Auto-splits a multi-image scan into separate documents. Best with ~1/8 in gaps and a uniform background; `Alt`/`Option` + command separates only the selected image. |
| PDF Presentation | `File > Automate > PDF Presentation` | Builds a multipage PDF or slide show from chosen/open files; supports reordering and duplication; output is a generic PDF (rasterised on reopen). |
| Contact Sheet II | `File > Automate > Contact Sheet II` | Builds a contact sheet from a folder/Bridge images: document size/mode, `Flatten All Layers`, thumbnail rows/columns, `Place` order, `Use Auto-Spacing`, `Rotate For Best Fit`, `Use Filename As Caption` (font/size). |
| Picture Package | `File > Automate > Picture Package` | Optional plug-in: lays multiple copies/images on a page; source via File/Folder/drag; page size/layout/resolution/mode; flatten option; labels; `Edit Layout` for custom layouts saved as text presets. |
| Photomerge | `File > Automate > Photomerge` | Stitches overlapping photos. Source `Files`/`Folders`; Layout `Auto`, `Perspective`, `Cylindrical`, `Spherical`, `Collage`, `Reposition`; options `Blend Images Together`, `Vignette Removal`, `Geometric Distortion Correction`. Produces one multi-layer document with layer masks. |
| Merge To HDR Pro | `File > Automate > Merge To HDR Pro` | Merges exposure-bracketed images into 8/16/32-bpc HDR; optional auto-align; `Manually Set EV` when metadata is missing; 32-bit white-point preview slider; 16/8-bit tone-mapping `Local Adaptation` (Edge Glow, Tone/Detail, Color, Toning Curve) or `Equalize Histogram`; save/load presets. |
| Web Photo Gallery | `File > Automate > Web Photo Gallery` | Optional plug-in: generates an HTML gallery (home page + pages + thumbnails + JPEGs) from images/folders using HTML templates/styles; options for General (extension, UTF-8, width/height, metadata), Banner, Large Images, Thumbnails, Colors, Security. |

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `File > Automate > Batch…` | dialog | — | Primary batch entry. |
| `Bridge > Tools > Photoshop > Batch` | menu | — | Batch from Bridge. |
| `File > Scripts > Image Processor…` | dialog | — | Action-free batch converter (script-provided in CS6). |
| `File > Automate > Conditional Mode Change…` | dialog | — | Records a step (`AUTO-001`). |
| `File > Automate > Crop And Straighten Photos` | menu | — | No dialog. |
| `File > Automate > PDF Presentation…` | dialog | — | |
| `File > Automate > Contact Sheet II…` | dialog | — | |
| `File > Automate > Picture Package…` | dialog | — | Optional plug-in. |
| `File > Automate > Photomerge…` | dialog | — | |
| `File > Automate > Merge To HDR Pro…` | dialog | — | Two-stage dialog. |
| `File > Automate > Web Photo Gallery…` | dialog | — | Optional plug-in. |
| Batch/droplet `Set`/`Action` combos | widget | — | Populated from the Actions panel. |

## Parameters & ranges

`Batch`/droplet options (the shared set; `BatchOptions` scripting names in parentheses):

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Set | enum | current set | loaded action sets | Required. |
| Action | enum | current action | actions in set | Required. |
| Source | enum | Folder | Folder / Import / Opened Files / Bridge (`from`) | |
| Folder | path | — | any readable | With `Choose`. |
| Include All Subfolders | bool | off | on/off | |
| Override Action "Open" Commands | bool | off | on/off (`overrideOpen`) | |
| Override Action "Save As" Commands | bool | off | on/off (`overrideSave`) | Action must contain `Save As`. |
| Suppress Color Profile Warnings | bool | off | on/off (`suppressProfile`) | |
| Suppress File Open Options Dialogs | bool | off | on/off (`suppressOpen`) | Recommended for Camera Raw. |
| Destination | enum | None | None / Save And Close / Folder (`destination`) | |
| Destination Folder | path | — | any writable (`destinationFolder`) | Folder destination only. |
| File Naming fields | list | `[Document Name]` | document name, serial number, serial letter, date, custom text | Must include ≥1 unique field. |
| Starting Serial Number | int | `1` | ≥1 (`startingSerial`) | |
| Compatibility | bool | on | Windows / macOS / UNIX (`macintoshCompatible`, `unixCompatible`, `windowsCompatible`) | |
| Error | enum | Stop For Errors | Stop For Errors / Log Errors to File (`errorFile`) | |

`Image Processor` options: process `Open Files`/`Folder`; save location; `Open First Image To Apply Settings`; `Save As JPEG` (quality `0`–`12`, `Resize To Fit` W/H, `Convert Profile To sRGB`), `Save As PSD` (`Maximize Compatibility`), `Save As TIFF` (`LZW Compression`); `Run Action` (set/action); `Copyright Info`; `Include ICC Profile`.

`Contact Sheet II`: `Use` source; document width/height/resolution/mode; `Flatten All Layers`; `Place` across/down; columns; rows; `Use Auto-Spacing` (or V/H spacing); `Rotate For Best Fit`; `Use Filename As Caption` + font/size.

`Photomerge`: source Files/Folders; Layout `Auto|Perspective|Cylindrical|Spherical|Collage|Reposition`; `Blend Images Together`, `Vignette Removal`, `Geometric Distortion Correction`.

`Merge To HDR Pro`: bit depth `8|16|32`; `Attempt To Automatically Align Source Images`; 32-bit white-point slider; 16/8-bit `Local Adaptation` (Edge Glow Radius/Strength, Gamma, Exposure, Detail, Shadow, Highlight, Vibrance, Saturation, Toning Curve) or `Equalize Histogram`; save/load preset.

`PDF Presentation`: file list (add/reorder/duplicate), `Output` and `Presentation` options.

`Web Photo Gallery`: General (extension `.htm`/`.html`, UTF-8 URL, width/height attrs, preserve metadata), Banner, Large Images, Thumbnails, Colors, Security (see Help; values not exhaustively captured here).

## Algorithms & pipeline

Behavioral parity only; Adobe's batch engine is closed. The observable pipeline:

```text
BatchJob { action, source, destination, naming, overrides, errors }:
  inputs = resolve(source)                      # folder(+recurse) | opened docs | import | bridge
  for i, input in inputs:
    if cancelled: break
    doc = open(input, suppress_open_dialogs, suppress_profile_warnings)
    if override_open and action has Open: use doc, ignore action's baked-in open file
    player.run(action, doc, displayDialogs=NO)  # AUTO-001
    if action has Save As:
        if override_save: name = apply_file_naming(i); path = destination(doc, name)
        save(doc, path, options_from_action)
    close(doc)
    on error: stop() or log(error_file) per policy
  report progress; end message for logged errors
```

- Batch runs with dialogs suppressed; any modal step in the action uses recorded values (`displayDialogs = NO`, `AUTO-010`).
- `Image Processor` is a different pipeline: no action required; per input it applies (optionally) Camera Raw settings, resizes, converts profile, adds metadata, and writes the selected format folders (`JPEG/`, `PSD/`, `TIFF/`).
- `Photomerge` = feature matching + bundle-adjust/alignment + projective/cylindrical/spherical/collage warping + seam-blended multi-layer output (`05-layers/*`). Exact algorithm TBD; only CS6-documented layout semantics are asserted here.
- `Merge To HDR Pro` = exposure-weighted radiance merge into 32-bit float + optional alignment, then the documented 8/16-bit tone-mapping operators (`04-image-ops/32-bit-hdr`).
- `Contact Sheet II` = grid layout of scaled thumbnails + optional caption text, as layers or flattened.
- `PDF Presentation` = paginate images into a generic PDF (`10-workflow-io/export-formats`).
- `Web Photo Gallery` = template-driven HTML/JPEG generation (styles + tokens); output tree `index.htm(l)`, `images/`, `pages/`, `thumbnails/`.

## Rust module mapping

- `crate::automation::batch` — `BatchJob`, `BatchSource` (`Folder{path, recurse}`, `OpenedFiles`, `Import{device}`, `Bridge`), `Destination` (`None`, `SaveAndClose`, `Folder(PathBuf)`), `FileNaming` (ordered `NamingField` list), `ErrorPolicy` (`Stop`, `LogTo(PathBuf)`), `Overrides`. Executes via `crate::action::player` and `crate::io`.
- `crate::automation::image_processor` — `ImageProcessorJob { sources, outputs: JpegOpts|PsdOpts|TiffOpts, resize, convert_srgb, metadata, run_action }`.
- `crate::automation::automate` — the remaining generators: `contact_sheet`, `pdf_presentation`, `photomerge`, `merge_hdr`, `web_photo_gallery`, `crop_straighten`. Each takes typed options and emits a document or file tree.
- `crate::automation::naming` — `FileNaming` evaluation (document name, serial number/letter, date, custom text) with uniqueness enforcement.
- `crate::job` — scheduling/cancellation/progress shared with `01-architecture/threading-and-concurrency`; a batch is a `Job` with per-input sub-tasks and a cancel token.
- `crate::action::player`, `crate::command`, `crate::io` — reused; no batch-specific mutation path.

Boundary types: `BatchOptions` (shared with `AUTO-002`), `FileNaming`, `ErrorPolicy`, `Progress`, `CancelToken`.

## Qt6 component mapping

- `BatchDialog` / `CreateDropletDialog` — both embed the same `BatchOptionsWidget` (set/action, source, destination, overrides, naming, errors) so the two cannot diverge.
- `FileNamingWidget` — ordered token fields (`QComboBox` + add/remove/reorder), starting serial, compatibility checkboxes.
- `BatchProgressDialog` — `QProgressDialog` fed by `crate::job`; Cancel → `CancelToken` → clean abort.
- `AutomateMenu` — built into `QMenuBar`; optional modules (`Picture Package`, `Web Photo Gallery`) are hidden unless installed, matching CS6's optional downloads.
- `ContactSheetDialog`, `PdfPresentationDialog`, `PhotomergeDialog`, `MergeHdrDialog` (two-stage: source list, then preview + tone map), `ImageProcessorDialog`.
- `ErrorLogView` — shows the error log after a `Log Errors to File` run.

Widgets, not QML: all are document-modal dialogs and progress utilities over `QMainWindow`.

## Data-model impact

- **New persisted types (outside PSD):** `BatchOptions` (with preferences, for last-used values); `DropletSpec` (`AUTO-002`) embeds a subset. `FileNaming`/`ErrorPolicy` are plain config.
- **Documents:** each input opens as a normal document; the batch does not add document-model fields. `Merge To HDR Pro` produces a 32-bit float document (`04-image-ops/32-bit-hdr`); `Photomerge` produces a multi-layer document with masks; `Contact Sheet II`/`Picture Package` produce layered or flattened documents.
- **Undo:** batch runs are not user-interactive; per-file edits need no interactive history (history can be minimised for speed). When a batch runs in-process with the GUI, each file's edits follow the standard history policy; no batch-level undo record.
- **XMP/metadata:** `Image Processor` `Copyright Info` overwrites IPTC copyright; `Web Photo Gallery` `Preserve All Metadata` controls metadata emission. Batch otherwise preserves file metadata per the format spec.
- **Serialization:** batch/droplet config never enters PSD/PSB.

## Edge cases

- **Action contains no `Save As`:** `Override Save As` has nothing to override and files are not saved; report it (CS6 note).
- **Action contains an `Open` for a baked-in file:** without `Override Open`, every iteration re-opens that file; the source files are silently ignored. Reproduce the warning.
- **Filename double-save:** if output is renamed in Batch but `Override Save As` is off, the file may be saved twice (new name in the batch folder, original name in the action's folder) — document and detect.
- **Serial uniqueness:** a naming scheme with no unique field overwrites files; require at least one unique token.
- **Missing/partial Camera Raw metadata:** `Merge To HDR Pro` requires `Manually Set EV`; batch Camera Raw needs `Suppress File Open Options Dialogs` or it stalls on hidden dialogs.
- **Import source without multi-document support:** a third-party acquire plug-in may fail under batch/actions; surface the plug-in name and the doc note.
- **Huge batches / PSB inputs:** cancellation must be responsive; memory is bounded per open document; process serially by default (parallel only with the documented thread budget in `01-architecture/threading-and-concurrency`).
- **8/16/32-bit and CMYK/Lab:** format writers must honour the document mode; `Image Processor` sRGB conversion is explicit.
- **Nested folders → different formats:** only works with `Save And Close` + `Override Save As`, relying on the action's recorded options (CS6 procedure).
- **Error-file logging:** the error log path must be writable; if not, fall back to `Stop For Errors` with a diagnostic.
- **Optional modules absent:** `Picture Package`/`Web Photo Gallery` menu items are hidden when the module is not installed, exactly as the optional CS6 plug-ins.
- **Disk full / write failure:** must not leave a partially written file where a rename-based atomic write is possible.

## Parity acceptance criteria

1. Given a folder and an action with a `Save As`, `Batch` with `Destination = Folder` + `Override Action "Save As" Commands` writes one file per input under the naming scheme, retaining the action's save options (e.g. JPEG quality).
2. Given `Include All Subfolders`, nested files are processed; without it, only top-level files are.
3. Given `Override Action "Open" Commands` and an action with an `Open` step, the batch-selected files are processed; without the override, only the baked-in file is (documented CS6 behaviour).
4. Given `Destination = Save And Close`, originals are overwritten in place; `None` leaves files unsaved when the action has no save step.
5. Given a naming scheme with a serial number starting at `N`, files are numbered `N`, `N+1`, …; a scheme with no unique field is rejected.
6. Given `Error = Log Errors to File` and an unreadable input among several, the run continues and writes an error file; `Stop For Errors` halts with a message.
7. Given `Image Processor` on a PSD/JPEG folder with JPEG+PSD+TIFF selected, it writes `JPEG/`, `PSD/`, `TIFF/` subfolders with the chosen options and both `ImageProcessor` outputs match the UI command's result within tolerance.
8. Given a bracket set, `Merge To HDR Pro` produces a 32-bit float document whose radiance is within the tone-map tolerance of the expected merge, and an 8/16-bit export applies the chosen tone mapper.
9. Given overlapping photos, `Photomerge` with a given Layout produces a multi-layer document with layer masks and matching overlap; layout choice changes the warp as documented.
10. Given a folder and `Contact Sheet II` with columns/rows/`Use Filename As Caption`, the sheet contains the expected grid and caption layers.
11. Given `PDF Presentation` with duplicate/reordered entries, the output PDF has the same page order/duplication and reopens rasterised.
12. Given `Web Photo Gallery` output, the destination contains `index.htm(l)`, `images/`, `pages/`, and `thumbnails/`, and the gallery opens without broken links.
13. Given a running batch, pressing Cancel aborts after the current sub-task, leaves no half-written output where atomic write is possible, and frees open documents.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — *Processing a batch of files*: *Convert files with the Image Processor*, *Process a batch of files*, *Batch and droplet processing options*; *Contact Sheets and PDF Presentations in CS6* (Contact Sheet II, PDF Presentation); *Photomerge* (source/layout/options, 360° Extended procedure); *Merge images to HDR* (Merge To HDR Pro, 32-bit and 16/8-bit options); *Picture packages and contact sheets* (Picture Package, optional plug-ins); *Creating web photo galleries* (Web Photo Gallery, options, output tree); *Crop and straighten scanned photos*; *Adding a conditional mode change to an action*. Fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-JavaScript-Ref.pdf` — `Application.batch`, `BatchOptions` properties, deprecated `makeContactSheet`/`makePDFPresentation`/`makePhotoGallery`/`makePhotomerge`/`makePicturePackage`. Community mirror; fetched via `curl` + `pdftotext`.
- Cross-reference `docs/09-automation/actions.md` (`AUTO-001`), `docs/09-automation/droplets.md` (`AUTO-002`), `docs/09-automation/rust-scripting-replacement.md` (`AUTO-010`), and `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`).
- `https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/` — generic PDF/PSD/TIFF save-option context (audience note); Actions format context. Fetched via `curl`.

## Open questions

- **Photomerge algorithm.** CS6 documents layout names and options but not the matcher/bundle-adjuster/blender. "Behavioral parity only, algorithm TBD"; resolve by comparing against CS6 output and published panoramic-stitching literature.
- **HDR merge math.** The radiance-merge weighting and tone-mapping operators are undocumented. Resolve against CS6 results (tolerance harness) and the `04-image-ops/32-bit-hdr` spec.
- **Error-file format/path.** Not documented; define a Kooka Pictura log format or recover it from a CS6 run.
- **`Web Photo Gallery` full option set and HTML templates.** Only partial options captured; the style templates are Adobe assets and cannot be copied (`00-overview/licensing-and-provenance`). Decide whether to ship independently authored styles.
- **`Picture Package` custom layout file format.** Described as text presets in the `Presets/Layouts` folder; the exact format was not retrieved.
- **`Image Processor` Camera Raw setting application.** The temporary-settings semantics ("the image's current camera raw settings are used unless changed") need a precise model in `06-filters/camera-raw-filter`.
- **Parallelism policy.** Whether batch may process files concurrently, and at what memory cost, is deferred to `01-architecture/threading-and-concurrency`.
- **Deprecated `Application.make*` methods.** CS6 marks `makeContactSheet`/`makePDFPresentation`/`makePhotoGallery`/`makePicturePackage` deprecated and `makePhotomerge` superseded by a `Photomerge.jsx` helper; whether to expose scripting shims is open.
- **Acceptance tolerances.** The "within tolerance" wording needs per-filter/per-operation numeric tolerances from the owning specs.
