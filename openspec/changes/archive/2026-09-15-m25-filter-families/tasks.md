## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m25-filter-families.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze the four enums, the 29 `Filter` variants, module paths, and validation ranges in `design.md`
- [x] 1.4 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Interface freeze in `lib.rs` (sub-agent, Rust)

- [x] 2.1 Add `StrokeDirection`, `LightDirection`, `HalftoneType`, `GrainType` public enums in `lib.rs`
- [x] 2.2 Add the 29 `Filter` variants with the frozen typed fields
- [x] 2.3 Add `apply` dispatch arms for every new variant
- [x] 2.4 Extend validation to every variant per the frozen ranges (boundary accepts, out-of-range rejects; `OilPaint` rejects non-finite floats)
- [x] 2.5 `cargo test -p pictura-filters` compiles; clippy clean

## 3. Wave 1: family kernels (parallel sub-agents, Rust)

### 3a Brush Strokes

- [x] 3a.1 `brush_strokes.rs`: Accented Edges, Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines, Sumi-e (edge/orientation + directional stroke + tonal gate)
- [x] 3a.2 Spatter and Sprayed Strokes (`StrokeDirection`, seeded scatter)
- [x] 3a.3 Tests: effect non-empty, alpha preserved, boundary accept / out-of-range reject, Crosshatch Strength passes monotonic, direction changes output, Spatter/Sprayed determinism
- [x] 3a.4 `cargo test -p pictura-filters`, clippy clean

### 3b Sketch

- [x] 3b.1 `sketch/relief.rs`: Bas Relief, Plaster, Note Paper (height field + `LightDirection` emboss)
- [x] 3b.2 `sketch/paper.rs`: Chalk & Charcoal, Charcoal, Conté Crayon, Reticulation, Torn Edges, Water Paper (seeded stroke/paper renderers)
- [x] 3b.3 `sketch/mod.rs`: Chrome, Graphic Pen (`StrokeDirection`), Halftone Pattern (`HalftoneType`), Photocopy, Stamp
- [x] 3b.4 Foreground/background as explicit fields; fg == bg safe; Conté Crayon reuses `TextureOptions`
- [x] 3b.5 Tests per spec scenarios (Bas Relief light direction, Halftone type/size/contrast, Photocopy edge-only dark areas, Note Paper emboss+grain, fg/bg identity, determinism)
- [x] 3b.6 `cargo test -p pictura-filters`, clippy clean

### 3c Texture

- [x] 3c.1 `texture.rs`: Craquelure, Mosaic Tiles, Patchwork, Stained Glass (cellular/tile + relief)
- [x] 3c.2 `Grain` with all 10 `GrainType` variants; Sprinkles/Stippled use the background colour; seeded
- [x] 3c.3 `Texturizer` via the shared `TextureOptions`
- [x] 3c.4 Tests: 10 grain types distinct, tile/cell size monotonic, Lighten Grout lifts grout, foreground border, texture options change output, degenerate sizes safe, determinism
- [x] 3c.5 `cargo test -p pictura-filters`, clippy clean

### 3d Oil Paint

- [x] 3d.1 `oil_paint.rs`: edge-aware directional smoothing + relief/Blinn-Phong shading (CPU behavioural model)
- [x] 3d.2 Six float controls; `angular_direction` 0..=360; reject non-finite
- [x] 3d.3 Tests: non-empty effect, Stylization/Cleanliness/Scale/Bristle Detail monotonic, angle rotates lighting, Shine raises specular, determinism, non-finite reject, alpha preserved
- [x] 3d.4 `cargo test -p pictura-filters`, clippy clean

## 4. App mapping and self-test (sub-agent)

- [x] 4.1 `filter_from_kind` maps the 29 new kinds to in-range defaults
- [x] 4.2 Extend `--self-test` (exit codes 68–69): apply representative filters from each family through the bridge and assert the image changes and alpha is preserved
- [x] 4.3 Build green; prior self-tests still exit 0

## 5. Close-out

- [x] 5.1 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [x] 5.2 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [x] 5.3 Archive the change and commit
