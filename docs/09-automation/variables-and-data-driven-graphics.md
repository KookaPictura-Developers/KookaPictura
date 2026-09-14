# Variables and Data-Driven Graphics

- **Spec ID:** `AUTO-011`
- **Status:** `Draft`
- **Parity tier:** `Core` — the Variables dialog, data sets, CSV/tab import, and data-set export are in both CS6 Standard and Extended.
- **New in CS6:** `No` — data-driven graphics (variables/data sets) date from CS2 and are unchanged in CS6. The CS6 Help notes GoLive recognises text and visibility variables but not pixel replacement; GoLive is not an Kooka Pictura concern.
- **Depends on:** `AUTO-010` (scripting engine), `05-layers/layers-overview`, `10-workflow-io/document-lifecycle`, `01-architecture/file-formats`, `01-architecture/undo-history`, `11-cross-cutting/localization`.

## CS6 behavior

Data-driven graphics produce multiple versions of an image from one template. The template is a PSD whose changing elements live on separate layers; variables label which elements change, and data sets hold one value per variable per version.

General workflow (from the CS6 Help, "Creating data-driven graphics"):

1. Create the base graphic as a template; separate changing elements onto layers.
2. Define variables (which elements change).
3. Create or import data sets (one per version).
4. Preview each data set against the document.
5. Generate the graphics by exporting them as PSD files.

### Variable types

| Type | Effect | Layer constraint |
|---|---|---|
| Visibility | Shows or hides the layer's content. | Any layer except the Background layer. |
| Pixel Replacement | Replaces the layer's pixels with pixels from another image file. | Any layer except the Background layer. |
| Text Replacement | Replaces a string of text in a type layer. | A type layer. |

Variables cannot be defined for the **Background** layer. Variable names must begin with a letter, underscore, or colon and cannot contain spaces or special characters except periods, hyphens, underscores, and colons. Several layers can be linked to the same variable (a link icon appears next to the Name menu); a layer with variables shows an asterisk in the Layer menu.

Pixel Replacement options:

- **Scale method:** `Fit` (fits inside the bounding box, may leave empty area), `Fill` (fills the box, may extend beyond it), `As Is` (no scaling), `Conform` (non-proportional stretch to the box).
- **Alignment:** a 3×3 alignment icon positions the image inside the box; not available for `Conform`.
- **Clip To Bounding Box:** clips parts outside the box; only available for `Fill` or `As Is`; not available for `Conform`.

### Data sets

A data set is a collection of variables and their associated data; one data set per version. `Image > Variables > Data Sets` (or the top-of-dialog pop-up / Next button) creates, edits, renames, and deletes data sets; arrow icons move between them. At least one variable must exist before the default data set can be edited. Per-variable data:

- Visibility: `Visible` / `Invisible`.
- Pixel Replacement: `Select File...` picks a replacement image; `Do Not Replace` leaves the layer as-is for that data set (and, per the Help, does **not** reset a previously applied replacement).
- Text Replacement: a text string in the `Value` box.

`Image > Apply Data Set` previews data sets in the document window and, on `Apply`, overwrites the base document with the chosen data set while leaving variables and data sets intact. Applying a data set is destructive to the current document state (undoable).

### Creating data sets in external files

Data sets can be authored in a text file or spreadsheet exported as CSV or tab-delimited text. Syntax:

```text
VariableName1<sep>VariableName2<sep> ... <sep>VariableNameN<nl>
Value1-1<sep>Value2-1<sep> ... <sep>ValueN-1<nl>
Value1-2<sep>Value2-2<sep> ... <sep>ValueN-2<nl>
...
Value1-M<sep>Value2-M<sep> ... <sep>ValueN-M<nl>
```

- `<sep>` is a comma (CSV) or tab; `<nl>` is line feed, carriage return, or both.
- The first line lists every variable name in column order; each following line is one data set.
- Visibility values are `"true"` / `"false"`.
- Spaces around the delimiter are trimmed; spaces inside a value are preserved, and leading/trailing spaces are preserved if the value is enclosed in double quotes.
- A double quote inside a value is escaped by doubling it (`""B""` → `B`).
- If `<sep>` or `<nl>` occurs in a value, the whole value must be double-quoted.
- Every variable defined in the PSD must appear in the file; a count mismatch is an error.
- Multi-line text in a text variable is created by quoting the value and inserting hard returns where breaks should occur.

Example (`FlowerShow.txt`):

```text
Variable 1, Variable 2, Variable 3
true, TULIP, c:\My Documents\tulip.jpg
false, SUNFLOWER, c:\My Documents\sunflower.jpg
false, CALLA LILY, c:\My Documents\calla.jpg
true, VIOLET, c:\My Documents\violet.jpg
```

Relative image paths are resolved against the text file's folder.

### Import and export

- `File > Import > Variable Data Sets` (or the Import button on the Data Sets page) loads the text file. Options: **Use First Column For Data Set Names** (name sets from column 1 instead of `Data Set 1`, `Data Set 2`, ...), **Replace Existing Data Sets**, and **Encoding** (default `Automatic`).
- `File > Export > Data Sets As Files` batch-exports every selected data set as a PSD: enter a base file name (with an optional naming scheme), choose a destination folder, choose which data sets, and click OK.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Variables > Define` | menu/dialog | — | Define/rename/remove variables per layer; Pixel Replacement options. |
| `Image > Variables > Data Sets` | menu/dialog | — | Create/edit/delete data sets; Import button. |
| Variables dialog — top pop-up / `Next` | dialog control | — | Switches Define ↔ Data Sets pages. |
| `Image > Apply Data Set` | menu/dialog | — | Preview and apply a data set; destructively overwrites the document. |
| `File > Import > Variable Data Sets` | menu/dialog | — | CSV/tab import; options above. |
| `File > Export > Data Sets As Files` | menu/dialog | — | Batch PSD output; base name, folder, set selection. |
| Variables dialog — Layer menu | dialog control | — | Layer picker; asterisk marks layers with variables; arrows navigate. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Variable name | string | derived from layer | must start `[A-Za-z_:]`; allowed chars letters digits `. - _ :`; no spaces | Field validation; duplicate names link layers. |
| Variable kind | enum | none | visibility, pixelReplacement, textReplacement | Multiple kinds per layer allowed. |
| Pixel scale method | enum | `Fit` | Fit, Fill, As Is, Conform | `As Is`/`Conform` disable other options per Help. |
| Pixel alignment | enum (3×3) | center | 9 positions | Disabled for `Conform`. |
| Clip To Bounding Box | bool | false | on/off | Only `Fill`/`As Is`. |
| Data set name | string | `Data Set N` | free text | `Use First Column For Data Set Names` overrides. |
| Visibility value | bool | current | true/false | `Visible`/`Invisible` in the dialog. |
| Text value | string | current text | any, incl. multiline | Newlines preserved. |
| Replacement file | path | none | image file | May be relative to the data file. |
| Delimiter | enum | auto-detect | comma, tab | Auto-detect from the header line. |
| `Use First Column...` | bool | false | on/off | Import option. |
| `Replace Existing...` | bool | false | on/off | Import option. |
| Encoding | enum | `Automatic` | Automatic + explicit codecs | Import option; BOM sniffing. |
| Export base name | string | `Untitled` | free text | Optional custom naming scheme. |

## Algorithms & pipeline

### Applying a data set

```text
apply_data_set(doc, data_set):
  for each (variable, value) in data_set:
      layer = resolve(variable.doc_ref)         # docRef = layer id, see XML model
      match variable.trait:
        visibility:
            set layer.visible = value            # one command
        text:
            set layer.text_item.contents = value # one command
        fileref (pixel replacement):
            if value is DoNotReplace: skip
            img = load_image(resolve_path(value, base_dir))
            placed = scale_and_align(img, method, align, clip, layer.bounds)
            replace layer pixels with placed       # one command
  commit as ONE history transaction labelled "Apply Data Set: <name>"
```

Applying overwrites the document's current state; it is therefore a single undoable transaction (matching CS6's "applying a data set overwrites your original document"). Previewing must not mutate the document until `Apply`.

### Generating files from data sets

```text
export_data_sets(doc, selected_sets, base_name, naming_scheme, out_dir):
  for i, ds in enumerated(selected_sets):
      snapshot = clone_document_state(doc)     # or re-open template from memory
      apply_data_set(snapshot, ds)             # no UI, displayDialogs = NO
      name = naming_scheme(base_name, ds, i)   # CS6 default: base_name + suffix
      save_as_psd(snapshot, join(out_dir, name + ".psd"))
      discard snapshot
```

Output format in CS6 is PSD (per the Help, "You can output images as PSD files"). Kooka Pictura preserves that default and may add other formats only as an explicit extension.

### CSV/tab parser

A small RFC-4180-flavoured parser is required: delimiter auto-detection (comma vs tab on the header line), doubled-quote escaping, quoted values that may contain the delimiter or newlines, and trailing whitespace trimming per the Help's rules. This is exactly a CSV parse, so the implementation must reuse one parser for both delimiters rather than hand-rolling a splitter (a naive `split(',')` breaks on the documented quoting rules).

### Data model (implied XML)

Although the Help presents variables/data sets as CSV, Photoshop persists them inside the PSD in **image resource blocks** (community analysis sources, marked inferred):

| Image resource ID | Name | Content |
|---|---|---|
| `7000` | `IMAGE_READY_VARIABLES` | XML: `variableSets` |
| `7001` | `IMAGE_READY_DATA_SETS` | XML: data sets |
| `7002` | `IMAGE_READY_DEFAULT_SELECTED_STATE` | default selected set |
| `7003`/`7004` | `IMAGE_READY_7_ROLLOVER_EXPANDED_STATE` / `IMAGE_READY_ROLLOVER_EXPANDED_STATE` | rollover state |
| `7005` | `IMAGE_READY_SAVE_LAYER_SETTINGS` | layer settings |
| `7006` | `IMAGE_READY_VERSION` | version |

The variables resource is an XML document in the namespace `http://ns.adobe.com/Variables/1.0/`:

```xml
<variableSets xmlns="http://ns.adobe.com/Variables/1.0/">
  <variableSet locked="none" varSetName="binding1">
    <variables>
      <variable varName="BorderCrop.Visible" trait="visibility" docRef="id('19910')"/>
      <variable varName="PixelVariable1" trait="fileref"
                placementMethod="fit" align="center" valign="middle"
                clip="false" docRef="id('13420')"/>
    </variables>
  </variableSet>
</variableSets>
```

Observed `trait` values: `visibility`, `fileref` (pixel replacement), and (for text) a text trait. Pixel placement attributes: `placementMethod` (`fit` observed; Fill/AsIs/Conform expected analogously), `align`/`valign` (`center`/`middle` observed), `clip` (`false` observed). `docRef` is `id('<layer id>')`. The exact data-sets XML element/attribute names were not retrieved; the shape above is inferred from the variables resource and must be verified against a real PSD before implementation.

Important negative finding: the CS6 scripting DOM does **not** appear to expose variables or data sets (community reports no `Document.variables`/`dataSets` object; the only route cited is editing image resource 7000, which Photoshop then rejected in a community test). Kooka Pictura treats a scripting API over variables as a **new** addition, not a parity requirement.

## Rust module mapping

- `crate::variables::model` — `VariableSet`, `Variable { name, kind, layer_id, placement: Option<PixelPlacement> }`, `VariableKind { Visibility, PixelReplacement, TextReplacement }`, `DataSet { name, values: BTreeMap<VariableId, DataValue> }`, `DataValue { Bool | Text(String) | File(PathBuf) | DoNotReplace }`, `PixelPlacement { method, align, valign, clip }`.
- `crate::variables::xml` — serde types for the 7000/7001 documents (`quick-xml` / `roxmltree`); round-trips the image resource block byte-for-byte where possible so Photoshop can still read the file.
- `crate::variables::csv` — delimiter-detecting, quote-aware reader/writer; validation against the document's variable list (count and names).
- `crate::variables::apply` — `apply_data_set`, `preview_data_set`, `export_data_sets`; each compiles to `crate::command` commands (one transaction).
- `crate::variables::registry` — maps variable kind ↔ layer capability; enforces "Background layer cannot have variables".
- Cross-links: `crate::formats::psd::image_resources` for keys 7000–7006; `crate::document` for layer/text/pixel access.

## Qt6 component mapping

- `VariablesDialog` (`QDialog`) with a `QStackedWidget`: Define page and Data Sets page; top pop-up / Next ↔ CS6.
  - Define page: `QComboBox` (layer, with `*` marker), checkboxes for the three kinds, `QLineEdit` (name, with validator), Pixel Replacement group (`QComboBox` method, 3×3 `AlignmentPicker`, `QCheckBox` clip).
  - Data Sets page: `QComboBox`/`QListView` of sets, `QLineEdit` (name), per-variable editor driven by a `QAbstractItemModel` (`VariableDataModel`), `Select File...` `QFileDialog`, Import button.
- `ApplyDataSetDialog` — set list with live preview (thumbnail via the render pipeline) and Apply/Cancel.
- `ImportDataSetDialog` — file picker + `Use First Column`, `Replace Existing`, encoding `QComboBox`.
- `ExportDataSetsDialog` (`Data Sets As Files`) — base name, `QFileDialog` folder, set selection list, naming-scheme field.
- Models (`QAbstractItemModel`): `VariableListModel`, `DataSetListModel`, `VariableDataModel`, so previews and edits stay consistent.

Widgets over QML: these are dense, keyboard-driven modal dialogs matching CS6; QML adds no value here.

## Data-model impact

- **New document-attached nodes (persisted in PSD image resources 7000/7001):** `VariableSet`, `Variable`, `DataSet`. They are metadata over layers referenced by generational layer ids; a layer delete invalidates its variables (policy below).
- **Command coverage:** new commands `SetLayerVisibility`, `SetTextContents`, `ReplaceLayerPixels` (already needed elsewhere) are reused; a `VariableBinding` is not itself a pixel operation.
- **Undo granularity:** one `Apply Data Set` or one `Export Data Sets As Files` document mutation = one history transaction. Individual variable edits in the dialog are not history states until applied (CS6 dialog behaves as a document-level edit on OK; Kooka Pictura may treat dialog OK as one state — see Open questions).
- **Serialization:** resource 7000/7001 XML is written in the Adobe namespace and kept in sync with the in-memory model; unknown resources round-trip untouched. XMP is not used for variables.
- **Layer id stability:** `docRef` uses the document's stable layer id, not the display name. Renaming a layer must not break the binding.

## Edge cases

- **Background layer:** variable definition disabled; the UI explains why.
- **Name validation:** leading char and allowed-character set enforced at entry; collisions link layers rather than error.
- **Count/name mismatch on import:** error listing which document variables are missing from, or extra in, the file.
- **Quoting/escaping:** delimiter or newline inside a value must be quoted; doubled quotes decode; unquoted ambiguity is a parse error with line number.
- **Relative paths:** resolved against the data file's directory; missing files fail that data set with a clear error and continue or abort per export policy.
- **Encoding:** BOM detection; Windows-1252 fallback under `Automatic`; explicit codecs honoured.
- **`Do Not Replace` semantics:** must not silently reset a previously applied pixel replacement (CS6 behaviour explicitly documented in the Help).
- **Apply is destructive:** must be one undo step; preview must not mutate.
- **Layer deleted after variables defined:** binding becomes dangling; the model should drop or quarantine it on load and report, not crash.
- **8/16/32-bit and CMYK/Lab:** replacement images are converted to the document's mode/bit depth with the document's conversion policy; no implicit mode change.
- **1-px / empty layers:** pixel replacement bounding box may be zero-area; define a documented fallback (skip or expand), never panic.
- **PSB/huge documents:** export loops over data sets and writes many large files; stream and check free space, don't hold all outputs in memory.
- **Empty data set:** allowed; applying it changes nothing; exporting still names a file.

## Parity acceptance criteria

1. Given a PSD with one text layer and one pixel layer, defining a Text Replacement and a Pixel Replacement variable and importing the example CSV produces data sets whose names and values match the file.
2. Given `Use First Column For Data Set Names`, data sets are named from column 1; otherwise `Data Set 1`, `Data Set 2`, ...
3. Given a file whose variable count does not match the document, import fails with an explicit count error and no partial import.
4. Given `Fit`, `Fill`, `As Is`, and `Conform` with the same replacement image and box, the placed-image bounds match the documented behaviour (Fit stays inside, Fill covers, As Is is 1:1, Conform has no alignment/clip).
5. Given `Apply Data Set`, the document reflects the set and exactly one history state named after the set exists; a prior undo restores the pre-apply image.
6. Given `File > Export > Data Sets As Files` for N selected sets, N PSD files are produced with the chosen base naming and each opens to the correct data set.
7. Given a variable name starting with a digit or containing a space, the dialog rejects it.
8. Given a quoted value containing a comma and one containing a newline, both round-trip through import/export unchanged.
9. Given the Background layer, the Define page offers no variable kinds.
10. Given a PSD written by Kooka Pictura with variables, Photoshop CS6 (where available) opens it and lists the same variables and data sets (binary-compat check; Adobe not available on Linux, so this is a compatibility-harness/Open-question criterion).

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — "Creating data-driven graphics" (pp. 637–641): general workflow, three variable types, Background-layer restriction, name rules, Pixel Replacement options, data-set definition, Apply/Preview, `Data Sets As Files`, external-file syntax and example, import options, encoding. Fetched via `curl` + `pdftotext`.
- `https://github.com/psd-tools/psd-tools/issues/375` — community analysis of `IMAGE_READY_VARIABLES`: namespace `http://ns.adobe.com/Variables/1.0/`, `variableSets`/`variableSet`/`variables`/`variable` structure, `varName`/`trait`/`docRef` attributes, `placementMethod`/`align`/`valign`/`clip` for `trait="fileref"`; report that editing the resource did not take effect in Photoshop (undocumented invariants).
- `https://raw.githubusercontent.com/psd-tools/psd-tools/main/src/psd_tools/psd/image_resources.py` — image resource key list: `IMAGE_READY_VARIABLES` 7000, `IMAGE_READY_DATA_SETS` 7001, `IMAGE_READY_DEFAULT_SELECTED_STATE` 7002, `IMAGE_READY_7_ROLLOVER_EXPANDED_STATE` 7003, `IMAGE_READY_ROLLOVER_EXPANDED_STATE` 7004, `IMAGE_READY_SAVE_LAYER_SETTINGS` 7005, `IMAGE_READY_VERSION` 7006.
- `https://community.adobe.com/t5/photoshop-ecosystem-discussions/access-image-variables-using-scripts/m-p/14994170` — community discussion indicating variables/data sets are not exposed in the CS6 scripting DOM (seen in search results; the thread was not opened in full).
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/Document.html` — CS6 `Document` surface (no variables/dataSets members in the mirrored class list), corroborating the scripting gap.
- `https://www.adobe.com/devnet-apps/photoshop/fileformatashtml` — Photoshop file-format specification (image-resource-block framing; resource ids above are community-mapped).

## Open questions

- **Data-sets XML shape.** The exact element/attribute names of `IMAGE_READY_DATA_SETS` (7001) were not retrieved. Resolve by dumping a real CS6 PSD with data sets (or reading psd-tools' handling) and recording the schema.
- **Why editing resource 7000 did not stick.** The community test wrote valid-looking XML and Photoshop ignored it. There may be checksums, ordering, or companion resources (7002/7005). Resolve with a controlled experiment on a real PSD.
- **Scripting API.** Whether to expose variables/data sets in the Kooka Pictura DOM as a deliberate extension (`document.variables`, `document.dataSets`) even though CS6 did not. Resolve with user need.
- **Dialog undo semantics.** Whether OK in the Variables dialog is one history state or immediate per-field edits. Resolve against CS6 behaviour on a real install.
- **Output formats.** CS6 exports PSD only; whether to extend to PNG/JPEG/TIFF is deferred.
- **`Data Sets As Files` naming scheme.** The exact CS6 default suffix/naming options (Help refers to "your own file-naming scheme") need capture from a real dialog.
- **Relationship to `10-workflow-io/bridge-and-interop`.** GoLive's partial variable support is obsolete; confirm no modern interop requirement.
- **Large-batch memory strategy.** Exporting hundreds of sets needs a streaming/clone strategy; resolve against `01-architecture/performance-targets`.
