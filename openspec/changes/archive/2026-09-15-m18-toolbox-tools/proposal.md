## Why

The shell now has a command registry, a tabbed document area, and a working file
lifecycle, but there is no way to interact with pixels: the canvas only pans and
zooms, and `docs/02-ui-ux/toolbox-and-options-bar.md` and `docs/03-tools/` have
no counterpart in code. M18 introduces the tool layer — a registry with an active
tool, a Tools panel and a context-sensitive options bar — and the first eight
tools, plus the small engine operations those tools need.

## What Changes

- Add a **tool framework**: a tool registry with a stable `ToolId`, an active
  tool, letter shortcuts, per-tool cursor and status hint, a Tools dock, and a
  context-sensitive **options bar** whose contents change with the active tool.
- Add **shape selection tools**: Rectangular/Elliptical Marquee, Lasso, and
  Quick Selection, with New/Add/Subtract/Intersect combine modes. This requires
  rectangle, ellipse, and polygon coverage rasterizers in `pictura-select`
  (Magic Wand already exists).
- Add **canvas tools**: Move (translate the active pixel layer), Crop (a crop
  region committed on Enter, destructive to the document for now), Eyedropper
  (sample a pixel), plus Hand and Zoom as first-class tools (their behavior
  already exists in the canvas widget but is not tool-driven).
- Add the engine operations those tools call: document crop, layer translation,
  pixel sampling, and the selection rasterizers.
- Extend `--self-test` with tool checks: tool switching, marquee/lasso/quick
  selection coverage and combine modes, crop, layer move, and eyedropper.

Out of scope: brush/paint tools, transform/warp, clone/heal, text/vector/shape
tools, Quick Mask and Refine Edge, non-destructive crop regions, selection
marching-ants for arbitrary masks (M18 shows a rubber band during the drag and
the committed selection bounds), and the Color panel (the Eyedropper records a
foreground colour on the frame pending M19).

## Capabilities

### New Capabilities

- `tool-framework`: the tool registry, active-tool state, letter shortcuts, the
  Tools dock, the per-tool options bar, cursor and status hints, and the canvas
  event routing that dispatches pointer input to the active tool.
- `shape-selection-tools`: Rectangular/Elliptical Marquee, Lasso, and Quick
  Selection over the existing selection model, including the coverage
  rasterizers and the combine modes.
- `canvas-tools`: Move, Crop, Eyedropper, Hand, and Zoom behavior, and the
  engine operations they require (document crop, layer translation, sampling).

### Modified Capabilities

_None._

## Impact

- Crates: `pictura-select` gains shape rasterizers and Quick Selection;
  `pictura-render` (document ops) gains crop and layer translation; `pictura-app`
  gains the tool framework, Tools dock, options bar, and canvas event plumbing;
  the cxx-qt bridge gains the tool operations and sampling.
- Build: `CMakeLists.txt` gains the tool sources. No new dependencies.
- Verification: `--self-test` grows tool checks and exit codes; existing
  engine/PSD behavior is unchanged.
