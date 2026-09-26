# ExtendScript API Surface

- **Spec ID:** `AUTO-005`
- **Status:** `Draft`
- **Parity tier:** `Core` for the surveyed subset; several classes are `Extended-only` (marked), and the AppleScript/VBScript bindings are `Non-goal (Linux)`. This is a **survey** of the CS6 JavaScript object model, not a reproduction of the reference.
- **New in CS6:** `Changed` — artboards were added to the DOM in CS6 (introduced to Photoshop in CS6); the tool-preset name is scriptable and a document returns a guides array; otherwise the object model is CS5-equivalent, and the interpreter remains ECMA-262 3rd ed. + E4X.
- **Depends on:** `AUTO-010` (rust-scripting-replacement), `AUTO-004` (script-events-and-jsx), `ARCH-011` (plugin-and-scripting-abi), `01-architecture/document-model`, `01-architecture/undo-history`.

## CS6 behavior

The Photoshop DOM is a **containment hierarchy**: objects are identified partly by their container. The `Application` root contains a `Documents` collection; a `Document` contains `ArtLayers`, `Layers`, `LayerSets`, `Channels`, `HistoryStates`, `PathItems`, `LayerComps`, `ColorSamplers`, `CountItems`, `Guides`, and a `Selection`. Collections group classes (`ArtLayers`, `Channels`, `ColorSamplers`, `CountItems`, `Documents`, `Layers`, `LayerComps`, `LayerSets`, `HistoryStates`, `Notifiers`, `PathItems`, `PathPoints`, `SubPathItems`, `TextFonts`). JavaScript collections are **0-based**; VBScript collections are 1-based.

This spec records the class hierarchy and the key members of each class so the Kooka Pictura DOM subset (`AUTO-010`) can be scoped. Property-by-property completeness is explicitly *not* in scope; the full list is in the CS6 JavaScript Scripting Reference.

### Containment hierarchy (key classes)

```text
Application (app)
 ├─ Notifier / Notifiers
 ├─ Preferences
 ├─ Document / Documents
 │   ├─ Selection
 │   ├─ Channel / Channels
 │   ├─ ArtLayer / Layer / LayerSet  (recursive LayerSet, per-set Layers/ArtLayers/LayerSets)
 │   │    ├─ TextItem            (only when ArtLayer.kind == LayerKind.TEXT)
 │   │    └─ LayerComp / LayerComps
 │   ├─ HistoryState / HistoryStates
 │   ├─ PathItem / PathItems
 │   │    └─ SubPathItem / SubPathItems
 │   │         └─ PathPoint / PathPoints
 │   ├─ DocumentInfo (File > File Info metadata)
 │   ├─ ColorSampler / ColorSamplers        (Extended)
 │   ├─ CountItem / CountItems              (Extended)
 │   └─ MeasurementScale                    (Extended; cannot be created)
 └─ (additional value/option classes: colors, save/open options, UnitValue, …)
```

`Application` and `Notifier` are the only classes that can be referenced without a containing `Document` (`AUTO-004`).

### `Application` (`app`)

Root of the model. Key **properties**: `activeDocument`, `documents`, `foregroundColor`, `backgroundColor`, `currentTool`, `displayDialogs`, `playbackDisplayDialogs`, `playbackParameters` (read-write `ActionDescriptor` for playback speed), `notifiers`, `notifiersEnabled`, `preferences`, `version`, `build`, `path`, `locale`, `colorSettings`, `fonts`, `recentFiles`, `freeMemory`, `measurementLog`, `name`, `systemInformation`, `scriptingVersion`, `scriptingBuildDate`, `windowsFileTypes`, `macintoshFileTypes`, `preferencesFolder`, `typename`.

Key **methods**: `open`, `openDialog`, `batch`, `doAction`, `executeAction`, `executeActionGet`, `charIDToTypeID`, `typeIDToCharID`, `stringIDToTypeID`, `typeIDToStringID`, `runMenuItem`, `showColorPicker`, `beep`, `bringToFront`, `refresh`, `refreshFonts`, `purge`, `load`, `putCustomOptions`, `getCustomOptions`, `eraseCustomOptions`, `featureEnabled`, `isQuicktimeAvailable`, `togglePalettes`, and the progress family `doProgress`, `doForcedProgress`, `doProgressTask`, `doProgressSubTask`, `doProgressSegmentTask`, `changeProgressText`. Deprecated (CS4/CS6): `makeContactSheet`, `makePDFPresentation`, `makePhotoGallery`, `makePicturePackage`, `makePhotomerge` (see `AUTO-003`).

### `Document` / `Documents`

The canvas. **Properties** include `activeChannels`, `activeHistoryBrushSource`, `activeHistoryState`, `activeLayer`, `artLayers`, `backgroundLayer`, `bitsPerChannel`, `channels`, `colorProfileName`, `colorProfileType`, `colorSamplers`, `componentChannels`, `countItems`, `fullName`, `guides`, `height`, `width`, `histogram` (256 entries; RGB/CMYK/Indexed only), `historyStates`, `info`, `layerComps`, `layers`, `layerSets`, `managed`, `measurementScale`, `mode`, `name`, `parent`, `path`, `pathItems`, `pixelAspectRatio` (`[0.100..10.000]`), `printSettings`, `quickMaskMode`, `resolution`, `saved`, `selection`, `xmpMetadata`, `typename`.

**Methods** include `add` (on `Documents`), `close`, `save`, `saveAs`, `exportDocument`, `duplicate`, `crop`, `resizeImage`, `resizeCanvas`, `rotateCanvas`, `flipCanvas`, `trim`, `revealAll`, `flatten`, `mergeVisibleLayers`, `rasterizeAllLayers`, `changeMode`, `convertProfile`, `splitChannels`, `paste`, `print`, `printOneCopy`, `importAnnotations`, `recordMeasurements`, `trap`, `suspendHistory` (single undo for the whole script), and `autoCount` (Extended).

### Layers

Two kinds: **`ArtLayer`** (image content) and **`LayerSet`** (a recursive group/folder). `Layer` is the shared base type.

`ArtLayer` **properties**: `allLocked`, `blendMode`, `bounds`, `fillOpacity`, `filterMaskDensity`/`filterMaskFeather`, `grouped`, `isBackgroundLayer`, `kind`, `layerMaskDensity`/`layerMaskFeather`, `linkedLayers`, `name`, `opacity`, `parent`, `pixelsLocked`, `positionLocked`, `textItem` (when `kind == LayerKind.TEXT`), `transparentPixelsLocked`, `vectorMaskDensity`/`vectorMaskFeather`, `visible`, `xmpMetadata`, `typename`.

`ArtLayer` **methods** fall into groups: adjustments (`adjustBrightnessContrast`, `adjustColorBalance`, `adjustCurves`, `adjustLevels`, `autoContrast`, `autoLevels`, `desaturate`, `equalize`, `invert`, `photoFilter`, `posterize`, `selectiveColor`, `shadowHighlight`, `threshold`, `matchColor`), the recorded-filter set (`applyAddNoise`, `applyAverage`, `applyBlur`, `applyBlurMore`, `applyClouds`, `applyCustomFilter`, `applyDeInterlace`, `applyDespeckle`, `applyDifferenceClouds`, `applyDiffuseGlow`, `applyDisplace`, `applyDustAndScratches`, `applyGaussianBlur`, `applyGlassEffect`, `applyHighPass`, `applyLensBlur`, `applyLensFlare`, `applyMaximum`, `applyMedianNoise`, `applyMinimum`, `applyMotionBlur`, `applyNTSC`, `applyOceanRipple`, `applyOffset`, `applyPinch`, `applyPolarCoordinates`, `applyRadialBlur`, `applyRipple`, `applySharpen`, `applySharpenEdges`, `applySharpenMore`, `applyShear`, `applySmartBlur`, `applySpherize`, `applyTwirl`, `applyUnSharpMask`, `applyWave`, `applyZigZag`, …), and structural operations (`clear`, `copy`, `cut`, `duplicate`, `link`/`unlink`, `merge`, `move`, `rasterize`, `remove`, `resize`, `rotate`, `translate`, `then` the layer effects `apply*` style methods).

`LayerSet` **methods**: `add`, `merge`; and it exposes its own `artLayers`, `layers`, `layerSets` collections so the hierarchy is recursive.

### `TextItem`

Created for a layer with `kind = LayerKind.TEXT`; properties cover `contents`, font/size/tracking/leading, colour, justification, warp, and position. Access via `artLayer.textItem`.

### `Selection`

The selected area of a layer/document. **Methods**: `select`, `selectAll`, `selectBorder`, `deselect`, `clear`, `copy`, `cut`, `fill`, `stroke`, `invert`, `grow`, `similar`, `expand`, `contract`, `feather`, `smooth`, `store` (save as channel), `load` (from channel), `makeWorkPath`, plus geometric transforms `rotate`/`resize`/`translate` and their `rotateBoundary`/`resizeBoundary`/`translateBoundary` variants.

### `Channel` / `Channels`

**Properties**: `color`, `histogram`, `kind` (component / masked / selected / spot), `name`, `opacity`, `parent`, `visible`, `typename`. **Methods**: `duplicate`, `merge` (spot → component), `remove`; `Channels` adds `add`, `getByName`, `removeAll`.

### `PathItem` / `SubPathItem` / `PathPoint`

A `PathItem` is a drawing object; `SubPathItem` carries the geometry; `PathPoint` carries each point. `PathItem` **methods**: `fillPath`, `strokePath`, `makeSelection`, `makeClippingPath`, `select`, `deselect`, `duplicate`, `remove`, `add`. Construction is via `PathPointInfo`/`SubPathInfo` arrays (each point has `anchor`, `leftDirection`, `rightDirection`, `kind`).

### `HistoryState` / `HistoryStates`

**Properties**: `name`, `snapshot`, `parent`, `typename`. The document's `activeHistoryState` is read-write, so setting it reverts the document. A history state can also be used as a fill source.

### `LayerComp` / `LayerComps`

Saved combinations of layer visibility/position/style. CS6 documents that certain operations (delete/merge a layer) can make a comp unrestorable.

### Color classes

`SolidColor` is the general colour definition and provides the type for `foregroundColor`/`backgroundColor`. **Properties**: `cmyk`, `gray`, `hsb`, `lab`, `rgb`, `model`, `nearestWebColor`. Concrete model classes: `RGBColor`, `CMYKColor`, `HSBColor`, `LabColor`, `GrayColor`, and `NoColor`; a `ColorModel`/`Color` enum selects. `SolidColor.isEqual(other)` compares colours. `Document` and `Application` colour properties are read-write; `setHexValue`/hex helpers exist for RGB.

### Action Manager classes

`ActionDescriptor`, `ActionList`, and `ActionReference` are the low-level Action Manager surface (not available in AppleScript) and are the escape hatch for behaviour the typed DOM does not cover (`AUTO-004`).

`ActionDescriptor` — a key→value record. Static property `count`; property `typename`. Methods (full inventory):

| Getter | Setter | Types |
|---|---|---|
| `getBoolean`, `getClass`, `getData`, `getDouble`, `getEnumerationType`, `getEnumerationValue`, `getInteger`, `getLargeInteger`, `getList`, `getObjectType`, `getObjectValue`, `getPath`, `getReference`, `getString`, `getType`, `getUnitDoubleType`, `getUnitDoubleValue`, `getKey` | `putBoolean`, `putClass`, `putData`, `putDouble`, `putEnumerated`, `putInteger`, `putLargeInteger`, `putList`, `putObject`, `putPath`, `putReference`, `putString`, `putUnitDouble` | boolean, class, raw data, double, enumerated, integer/large integer, list, object, path, reference, string, unit double |
| `hasKey`, `erase`, `clear`, `isEqual` | `fromStream`, `toStream` | serialization |

`ActionList` — an append-only list with the same getter/setter vocabulary (`putInteger`, `putEnumerated`, `putUnitDouble`, …); `ActionReference` — a reference to a property/class/name/index/identifier/offset with `putClass`, `putEnumerated`, `putIdentifier`, `putIndex`, `putName`, `putOffset`, `putProperty` and matching getters. Values are addressed by **runtime IDs** derived from four-char/string IDs via `charIDToTypeID`/`stringIDToTypeID`; `typeIDToCharID`/`typeIDToStringID` invert them.

### Strings, units, options, constants

- **Unit values:** `UnitValue` (number + unit) is returned by geometry properties and accepted by transform methods; unit constants (`Units.PIXELS`, `Units.INCHES`, …) are enums.
- **Option classes:** open/save/export options are classes passed to `open`/`saveAs`/`exportDocument` — e.g. `BMPSaveOptions`, `CameraRAWOpenOptions`, `DICOMOpenOptions`, `EPSOpenOptions`, `PDFOpenOptions`, `PhotoCDOpenOptions`, `RawFormatOpenOptions`, `BitmapConversionOptions`, `IndexedConversionOptions`, plus `BatchOptions`, `ContactSheetOptions`, `PresentationOptions`, `GalleryOptions`, `PicturePackageOptions` for the Automate commands (`AUTO-003`).
- **Enumerations/constants** define allowed values: `LayerKind`, `BlendMode`, `DialogModes` (`ALL`/`ERROR`/`NO`), `DocumentMode`, `BitsPerChannelType`, `ColorProfileType`, `ChangeMode`, `PurgeTarget`, `SaveOptions`, `ExportType`, `ReferenceFormType`, `DescValueType`, … The constant set is large; Kooka Pictura exposes the subset its commands implement.

### ExtendScript-runtime classes (not Photoshop DOM)

`File`, `Folder`, `Socket`, `XML` (E4X), `ExternalObject`, `$` (the ExtendScript global), and ScriptUI control classes are provided by the ExtendScript runtime, not Photoshop. Their Kooka Pictura treatment (sandboxed `File`/`Folder`, no `Socket`/`ExternalObject`/E4X, minimal ScriptUI) is in `AUTO-004`/`AUTO-010`.

## UI surface

The DOM itself has no UI. This survey maps to:

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| ExtendScript Object Model Viewer | external tool | — | CS6's browser for the DOM; Kooka Pictura provides a generated HTML/CLI reference instead. |
| Script console (new) | panel | — | Inspect/print DOM values; see `AUTO-004`. |
| `File > Scripts` | menu | — | Scripts using this API run here. |

## Parameters & ranges

Range-bearing values captured during the survey (illustrative, not exhaustive):

| Member | Type | Range / options | Notes |
|---|---|---|---|
| `ArtLayer.opacity` | number | `[0.0..100.0]` | percent |
| `ArtLayer.fillOpacity` | number | `[0.0..100]` | percent |
| `ArtLayer.layerMaskDensity` | double | `[0.0..100.0]` | |
| `ArtLayer.layerMaskFeather` | double | `[0.0..250.0]` | |
| `ArtLayer.filterMaskDensity`/`Feather`, `vectorMaskDensity`/`Feather` | double | `[0.0..250.0]` | |
| `Document.pixelAspectRatio` | number | `[0.100..10.000]` | |
| `Document.histogram` | array | 256 members | RGB/CMYK/Indexed only |
| `SolidColor.model` | enum | ColorModel | selects rgb/cmyk/hsb/lab/gray |
| `Application.displayDialogs` | enum | `ALL`, `ERROR`, `NO` | |
| `LayerKind` | enum | e.g. `TEXT`, … | gate for `textItem` |

## Algorithms & pipeline

This is a data/API survey; the execution model lives in `AUTO-010`/`ARCH-011`. The relevant mapping rules are:

- Each DOM object is a thin, stateless-for-calls handle over document state; **every mutation compiles to a command** on the shared command bus, so scripts, actions, and plug-ins share one undo path and one dialog-mode gate.
- Handles are generational identifiers (`OpDocId`/`OpLayerId`-style) that are invalidated on close/delete; a script must re-fetch after a mutation (`ARCH-011`).
- Collections are lazily materialised views over the document model; `getByName` and index access are resolved against the current model, and 0-based indexing is preserved for JS.
- `executeAction`/`executeActionGet` and `ActionDescriptor` reuse the same command objects the UI builds; the four-char ↔ string table is independent-creation (`ARCH-011` legal note).
- Option classes are plain typed structs; no document state is stored in them.

## Rust module mapping

Extends the `crate::script::dom` layout of `AUTO-010`:

- `crate::script::dom::application` — `Application` class; globals `app`, `$`.
- `crate::script::dom::document` — `Document`/`Documents`; exposes `activeHistoryState`, `suspendHistory`, `saveAs`/`exportDocument`, collections.
- `crate::script::dom::layer` — `Layer` base, `ArtLayer`, `LayerSet`, `TextItem`; typed accessors for the surveyed properties; group methods delegating to `crate::command`.
- `crate::script::dom::selection`, `channel`, `path`, `history`, `layer_comp` — the corresponding wrappers.
- `crate::script::dom::color` — `SolidColor`/`RGBColor`/`CMYKColor`/`HSBColor`/`LabColor`/`GrayColor`, model conversion via `crate::color`.
- `crate::script::actions` — `ActionDescriptor`/`ActionList`/`ActionReference` (shared with `crate::action`, `ARCH-011`).
- `crate::script::dom::options` — typed open/save/export option classes mapped to `crate::io` and `crate::automation`.
- `crate::script::dom::errors` — `NotImplementedError` naming the CS6 API for unimplemented members.

Boundary types are plain values (numbers, strings, `UnitValue`, `ActionValue`); no Rust borrow escapes a call.

## Qt6 component mapping

No dedicated widgets; this surface is consumed through `AUTO-004`'s components:

- `ScriptConsoleDock` — print/inspect DOM values and errors.
- `ScriptSecurityDialog` — permission gate for `File`/`Folder`-touching API.
- `ProgressProxy` — the `doProgress*`/`changeProgressText` bridge.
- `ScriptUI shim` (optional) — where DOM option/dialog classes surface as Qt dialogs.
- A generated **API reference** (HTML/`QTextBrowser`) replacing CS6's ExtendScript Object Model Viewer.

## Data-model impact

- No new persisted document fields: the DOM is a view over `crate::document` (`01-architecture/document-model`). Artboards (new in CS6) and guides already exist there.
- New in-memory wrapper types only: the `crate::script::dom::*` classes and the option/colour structs (mapped to existing core types).
- Undo: any mutating DOM call participates in the enclosing script/action transaction (`AUTO-004`); no per-property undo record.
- Serialization: DOM state is written by the normal PSD/XMP writers; the DOM layer adds nothing to the file format.

## Edge cases

- **0-based JS vs 1-based VBScript collections:** the JS DOM is 0-based; the VBScript DOM is 1-based. Only the JS surface is targeted, so indexing must never silently shift.
- **Read-only vs read-write:** many properties are read-only (`Document.mode`, `resolution`, `width`/`height`, `bounds`, `parent`, `typename`); writing them must raise, not no-op.
- **`textItem` gating:** valid only when `kind == LayerKind.TEXT`; otherwise a named error.
- **`LayerKind` on non-empty layers:** `kind` is writable only for an empty, non-background layer; other cases raise.
- **Extended-only members:** `autoCount`, `ColorSampler`, `CountItem`, `MeasurementScale` are Extended-edition; on the Standard surface they must report unsupported rather than silently succeed.
- **History/`activeHistoryState`:** reverting via the setter must be transactional and must not corrupt the history list; CS6 notes a first-history-state error before the app is front-most.
- **`histogram` mode gating:** invalid for modes other than RGB/CMYK/Indexed; must raise.
- **Large/PSB documents:** geometry returns `UnitValue`; `int32`-scale assumptions must not overflow at 300,000 px (`ARCH-011` ROI caveat).
- **8/16/32-bit and CMYK/Lab:** values are document-native; conversions are explicit (`crate::color`), never implicit.
- **Unimplemented member:** throws `NotImplementedError` with the CS6 name (`AUTO-010`), so migration gaps are visible.
- **Unit strings:** parsing/serialising `UnitValue` must round-trip CS6 units (`Units.*`).
- **XMP/`DocumentInfo`:** metadata is passed through, not reinterpreted; `Document.info` and `xmpMetadata` must agree with `10-workflow-io/file-info-and-metadata`.

## Parity acceptance criteria

1. Given `app.activeDocument.activeLayer.opacity = 50`, the active layer opacity is `50` and one history state is created (script-level).
2. Given `doc.artLayers.getByName("foo")` for an existing layer, the returned handle reports the same `name`, `bounds`, `kind`, and `visible` as the UI.
3. Given a text layer, `layer.textItem.contents = "Hi"` sets the text; on a non-text layer, `textItem` raises a named error.
4. Given `doc.activeHistoryState = savedState`, the document reverts to that state and the history list is consistent.
5. Given `app.selection.select(region)` then `selection.store(channel)`, a new channel of the expected kind is created; iterating `doc.channels` reflects it.
6. Given `charIDToTypeID` + `ActionDescriptor.putInteger` + `executeAction` for a supported event, the mutation matches the UI command within tolerance; `ActionDescriptor.toStream`/`fromStream` round-trips the descriptor.
7. Given `new RGBColor()` with `rgb.red = 255` etc., the applied `foregroundColor` matches the UI colour within 1 LSB.
8. Given a Standard-edition surface, an Extended-only member (`autoCount`) reports unsupported with a named error.
9. Given a read-only property write (`doc.mode = …`), the call raises rather than silently mutating.
10. Given a `.jsx` calling an unimplemented DOM method, the error names the CS6 method (`AUTO-010`).

## Sources

- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-JavaScript-Ref.pdf` — *Adobe Photoshop CS6 JavaScript Scripting Reference* (232 pp.): object descriptions and containment hierarchy, `ActionDescriptor`/`ActionList`/`ActionReference` full method tables, `Application`, `ArtLayer`, `Channel`, `CMYKColor`, `Document`, `HistoryState`, `LayerSet`, `PathItem`, `RGBColor`, `Selection`, `SolidColor`, `TextItem`, default-value ranges, Appendix A event IDs. Community mirror; fetched via `curl` + `pdftotext`.
- `https://archive.org/download/manual-photoshop-COMPLETO/Photoshop-CS6-%28Ingles%29.pdf` — same CS6 JavaScript Scripting Reference (alternate copy; used to confirm version/page count). Fetched via `curl` + `pdftotext`.
- `https://github.com/johnshopkins/adobe-scripts/raw/master/Photoshop/Photoshop-CS6-Scripting-Guide.pdf` — containment hierarchy, collection semantics (0-based JS / 1-based VBScript), Notifier/PathItem examples, Action Manager objects note ("not available in AppleScript"). Community mirror; fetched via `curl` + `pdftotext`.
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/Application.html` and sibling pages (`Document`, `ArtLayer`, `LayerSet`, `Selection`, `Channel`, `PathItem`, `HistoryState`, `SolidColor`, `ActionDescriptor`) — community mirror of the scripting reference used to cross-check property/method signatures. Fetched via `curl`.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — scripting chapter and What's New (artboards/DOM changes) context. Fetched via `curl` + `pdftotext`.
- Cross-reference `docs/09-automation/rust-scripting-replacement.md` (`AUTO-010`), `docs/09-automation/script-events-and-jsx.md` (`AUTO-004`), and `docs/01-architecture/plugin-and-scripting-abi.md` (`ARCH-011`).

## Open questions

- **Subset boundary.** Which classes/members the Kooka Pictura DOM implements (and which throw `NotImplementedError`) is not yet enumerated. Resolve by surveying real scripts (`AUTO-010`) and ranking by usage.
- **`Application.playbackParameters` keys.** Exposed as an opaque `ActionDescriptor`; its schema is undocumented (`AUTO-001` Open questions).
- **Option-class completeness.** The open/save/export option classes are numerous; the exact set needed for CS6 format parity is owned by `01-architecture/file-formats` / `10-workflow-io`.
- **Artboards in the DOM.** CS6 adds artboards to the UI; the corresponding JavaScript classes/members were not fully captured. Resolve from the CS6 reference (artboard/layer-section classes) and `05-layers/artboards`.
- **Guides/tool-preset additions.** The CS6 "guides array" and tool-preset name are noted in What's New but their exact API signatures were not extracted.
- **Extended-edition gating.** How Standard vs Extended surfaces report Extended-only members needs a policy (raise vs omit).
- **Object Model Viewer replacement.** The generated reference's format and update mechanism are undecided.
- **`UnitValue` precision and units.** Round-trip fidelity across all `Units.*` values is unverified.
- **Reference form constants.** `ReferenceFormType`/`DescValueType` and the four-char id table need provenance (`ARCH-011`).
