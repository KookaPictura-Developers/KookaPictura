# Mixer Brush Tool

- **Spec ID:** `TOOL-022`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the Mixer Brush was introduced in CS5 ("Extraordinary painting effects"), and CS6 carries it forward. CS6 does not document Mixer-Brush-specific changes; it gains the CS6 tip families (bristle, erodible, airbrush) and Brush Pose. Any claim that the CS6 Mixer Brush math differs from CS5 is unverified.
- **Depends on:** `01-architecture/rust-core-design.md`, `01-architecture/undo-history.md`, `03-tools/brush-and-pencil.md`, `07-color-painting/mixer-brush-engine.md`, `07-color-painting/bristle-brushes.md`, `05-layers/blend-modes.md`.

## CS6 behavior

The Mixer Brush simulates realistic painting: mixing colors on the canvas,
combining colors on the brush, and varying paint wetness across a stroke. It is
grouped with the Brush tool (click and hold the Brush to reveal it).

The CS6 Help describes a **two-well** brush model:

- a **reservoir** that stores the final color deposited on the canvas and has
  more paint capacity;
- a **pickup** well that receives paint only from the canvas and whose contents
  are continuously mixed with canvas colors.

Documented procedure:

1. Select the Mixer Brush tool.
2. **Load paint into the reservoir** by `Alt`/`Option`-clicking the canvas, or by
   choosing a foreground color. When loading from the canvas, the tip reflects
   the sampled area's color variation; selecting **Load Solid Colors Only** in
   the **Current Brush Load** pop-up gives a uniform tip color.
3. Choose a brush from the Brush Presets panel.
4. Set tool options. Common options are the same as other paint tools (mode,
   opacity, etc.). Mixer-Brush-specific controls:
   - **Current Brush Load swatch** — its pop-up has **Load Brush** (fill with
     reservoir color) and **Clean Brush** (remove paint). **Automatic Load** and
     **Automatic Clean** checkboxes perform these after each stroke.
   - **Preset pop-up** — popular combinations of Wet, Load, and Mix.
   - **Wet** — how much paint the brush picks up from the canvas; higher settings
     produce longer paint streaks.
   - **Load** — the amount of paint loaded in the reservoir; at low load rates
     strokes dry out more quickly.
   - **Mix** — the ratio of canvas paint to reservoir paint. At 100%, all paint
     is picked up from the canvas; at 0%, all comes from the reservoir. (Wet
     still determines how paints mix on the canvas.)
   - **Sample All Layers** — picks up canvas color from all visible layers.
5. Drag to paint, click + `Shift`-click for a straight line, or hold the button
   still to build up (airbrush behavior).

The **Clean brush after stroke** requirement is the `Clean` / automatic-clean
control above: paint accumulated in the brush is discarded so the next stroke
starts from the reservoir color (or an emptied brush), rather than carrying the
previous stroke's canvas paint forward.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel (Brush slot, hidden tool) | Tool | `B` + `Shift+B` cycle | Reveal by holding the Brush tool. |
| Options bar | Tool bar | n/a | Current Brush Load swatch, preset pop-up, Wet, Load, Mix, Flow, Sample All Layers, brush tip. |
| Current Brush Load pop-up | Pop-up | n/a | Load Brush / Clean Brush buttons; Load Solid Colors Only; automatic Load/Clean. |
| Preset pop-up | Pop-up | n/a | Wet/Load/Mix combinations. |
| Brush Presets panel | Dock | `Window > Brush Presets` | Preset tips, including bristle tips. |
| Brush panel | Dock | `Window > Brush` | Tip shape and dynamics for the mixer tip. |
| Shortcuts | Keyboard | number keys = Wet; `Shift`+number = Mix; `00` = Wet and Mix to 0 | Per the CS6 shortcut table. |
| Canvas | On-canvas | `Alt`/`Option`-click | Load paint from the canvas into the reservoir. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Wet | percent | not stated | 0 – 100 | Canvas pickup; higher = longer streaks. |
| Load | percent | not stated | 0 – 100 | Reservoir paint; low = stroke dries out. |
| Mix | percent | not stated | 0 – 100 | 100 = all canvas, 0 = all reservoir. |
| Flow | percent | 100 (inferred) | 0 – 100 | Rate of application; same meaning as other paint tools. |
| Sample All Layers | bool | off | on / off | Sample canvas color from all visible layers. |
| Load Solid Colors Only | bool | off | on / off | Uniform tip instead of sampled variation. |
| Auto Load | bool | off | on / off | Load reservoir before each stroke. |
| Auto Clean | bool | off | on / off | Clean the brush after each stroke. |
| Preset | enum | not stated | Dry / Moist / Wet / Very Wet variations (unverified names) | Applies a Wet/Load/Mix combination. |
| Mode | enum | Normal | CS6 paint-mode list | Shared paint modes. |
| Brush tip | preset | bristle tip (inferred default) | Bristle / standard / others | Bristle tips are the natural pairing. |

## Algorithms & pipeline

Adobe's Mixer Brush math is closed; this is a **behavioral-parity proposal**.

State the engine carries per stroke:

```text
MixerState {
  reservoir: Color,      // final deposited color load
  reservoir_amount: f32, // Load, 0..1
  pickup: Color,         // accumulated canvas paint
  wet: f32,              // 0..1
  mix: f32,              // 0..1
}
```

Per dab:

1. **Pickup.** Sample the canvas/composite under the dab (or all visible layers
   if Sample All Layers). With `Load Solid Colors Only`, sample to a single
   color; otherwise retain the sampled variation. Update `pickup` toward the
   sample proportional to `Wet`.
2. **Mix.** Choose the outgoing color as a blend of `pickup` and `reservoir`
   weighted by `Mix` (100% → canvas, 0% → reservoir).
3. **Load / dry-out.** Deposit a quantity proportional to `Load` (and `Flow`);
   decrement `reservoir_amount`. At low Load, the stroke visibly dries out.
4. **Wet streaks.** `Wet` also controls how far canvas colors are dragged along
   the stroke, producing longer streaks at higher values. *(inferred.)*
5. **Clean.** After the stroke, `Auto Clean` resets `pickup` (and optionally the
   reservoir) to prevent bleed into the next stroke.
6. **Commit.** The stroke accumulates in a sparse tile scratch buffer and commits
   on pointer release, exactly like the Brush tool.

## Rust module mapping

Design proposal.

- `pictura-paint::mixer` — `MixerEngine`, `MixerState`, and
  `MixerPreset { wet, load, mix }`; implements `begin_stroke / dab / clean /
  end_stroke`.
- `pictura-paint::brush` — tip footprint / bristle rendering reused from
  `TOOL-020`.
- `pictura-paint::sample` — canvas sampling (`ActiveLayer | AllVisible`) with
  adjustment-layer handling.
- `pictura-core::command` — `MixerStrokeCommand { layer, tip, preset, mode,
  sample_all_layers, load_solid_colors, dirty_tiles }`.

Crossing types: `LayerId`, `Rect`, `ColorSpace`, `BlendMode`, `TileDelta`,
`PixelBuffer`.

## Qt6 component mapping

- `MixerBrushOptionsBar` (`QWidget`) — Current Brush Load swatch + pop-up,
  preset pop-up, Wet/Load/Mix sliders, Flow, Sample All Layers.
- `BrushLoadPopup` (`QWidget`) — Load/Clean buttons, Load Solid Colors Only,
  automatic Load/Clean toggles; shows a reservoir/pickup color preview.
- `BrushPresetModel` — reused.
- `CanvasView` — cursor/HUD overlay; pixels stay in the GPU pipeline.

## Data-model impact

- **No PSD fields.** Only pixel tiles change.
- **Undo.** One state per completed stroke (`01-architecture/undo-history.md`);
  the brush's reservoir/pickup state is transient tool state, not history.
- **Persistence.** Mixer presets are tool presets, saved outside the document
  (`10-workflow-io/presets-manager.md`).
- **Sampling across layers** reads the composited result (or per-layer data), so
  the engine needs a read-only view of the visible composite that does not force
  a layer flatten.

## Edge cases

- **Hidden / non-pixel target layer.** Sampling all layers reads the composite;
  painting still targets one raster layer. On a non-pixel layer the command must
  be refused or rasterize explicitly.
- **Adjustment layers during sampling.** Sampled color should reflect the visible
  composite; define whether adjustment layers participate (CS6 "Ignore
  Adjustment Layers" exists on the Eyedropper/Sample control — confirm for the
  Mixer Brush).
- **Empty reservoir.** A freshly cleaned brush with no load deposits pickup
  color; behavior must match CS6 rather than painting nothing.
- **Dry-out.** Very low Load must produce a visibly drying stroke, not a shorter
  one that then resets silently.
- **Bitmap / Indexed / 32-bit modes.** Painting restrictions and the 32-bit mode
  subset apply as for the Brush tool.
- **Huge documents.** Sparse stroke buffer; stock brush sizes plus 5000 px tips
  must not force full-canvas copies.
- **Undo mid-stroke.** Atomic on release.
- **GPU unavailable.** CPU fallback must render bristle/mixer strokes.

## Parity acceptance criteria

1. Given Wet/Load/Mix at (0/100/0) and a foreground-color reservoir, the stroke
   paints essentially the reservoir color with no canvas pickup; at (100/100/100)
   it is dominated by canvas color.
2. Given increasing Wet with fixed Load/Mix, painted streaks are measurably
   longer (drag distance of canvas color along the stroke).
3. Given low Load, a stroke dries out over its length; given high Load, it does
   not.
4. Given `Clean Brush` (or Auto Clean) after a stroke, the next stroke does not
   carry the previous stroke's canvas paint.
5. Given `Load Solid Colors Only`, the deposited color is uniform across the
   stroke; deselected, it carries sampled variation.
6. Given `Sample All Layers`, colors from non-active visible layers are picked
   up; deselected, only the active layer is sampled.
7. Given a completed stroke, exactly one history state is created and its undo
   record references only touched tiles.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference: "Painting with the Mixer Brush"
  (pp. 419–420) for the reservoir/pickup model, load-from-canvas, Load Solid
  Colors Only, Current Brush Load (Load/Clean + automatic), preset pop-up,
  Wet/Load/Mix definitions, Flow, Sample All Layers; "What's new in CS5" (p. 28)
  placing "Extraordinary painting effects" (Mixer Brush + bristle tips) under CS5;
  shortcut table entries for number keys / `00` / `Alt+Shift`+number (p. 82 area).
- `https://html.duckduckgo.com/html/?q=Photoshop+mixer+brush+%22wet%22+%22load%22+%22mix%22+preset+dry+moist+CS5+CS6` —
  search results corroborating Wet/Load/Mix and Dry/Moist/Wet/Very Wet preset
  names (Bapu Graphics, PRO EDU, LensVid snippets). Search-results page;
  community sources, not authoritative for exact preset values.

## Open questions

- **Preset names and exact Wet/Load/Mix values** for the CS6 Mixer Brush presets
  are not in the Help PDF. *Resolves with:* a CS6 preset export or a screenshot
  of the preset pop-up.
- **Defaults** for Wet, Load, Mix, Flow, Mode, and the default tip are not
  stated. *Resolves with:* a CS6 options-bar capture.
- **Exact mixing/dry-out math**, streak-length dependence on Wet, and the
  reservoir-depletion curve are unspecified. *Resolves with:* pixel-level
  comparison against CS6 and an accepted tolerance.
- **Whether CS6 changed anything** in the Mixer Brush relative to CS5 is
  unverified; the CS6 Help does not list a Mixer-Brush change. *Resolves with:*
  a CS5 vs CS6 side-by-side.
- **Adjustment-layer participation** in Sample All Layers is unconfirmed.
  *Resolves with:* a CS6 experiment.
