## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m21-paint-engine.md` milestone brief
- [x] 1.2 Freeze the `pictura-paint` public types and the bridge stroke API in `design.md`
- [x] 1.3 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Wave 1: `pictura-paint` crate (sub-agent, Rust)

- [ ] 2.1 Add `crates/pictura-paint` to the workspace with `pictura-core` as its only dependency
- [ ] 2.2 `tip`: procedural round/elliptical coverage with hardness, roundness, angle, and an anti-aliasing ramp; an aliased (Pencil) mode
- [ ] 2.3 `spacing`: fixed-percent and velocity-driven resampling with residue carry-over and a minimum step
- [ ] 2.4 `stroke`: per-pixel coverage accumulation with the flow/opacity model and a per-stroke scratch buffer
- [ ] 2.5 `composite`: apply the scratch to a layer copy through `Normal`, `Dissolve`, `Behind`, `Clear`
- [ ] 2.6 `StrokeConfig` validation (size 1..=5000, hardness/opacity/flow 0..=100, roundness 0..=100, angle -180..=180)
- [ ] 2.7 Unit/property tests: spacing intervals, opacity cap, flow ordering, pencil aliasing, mode behavior, deterministic dissolve with a fixed seed
- [ ] 2.8 `cargo test -p pictura-paint`, `cargo clippy` clean

## 3. Wave 2: bridge and tool integration (sub-agent)

- [ ] 3.1 Bridge: `begin_paint(settings)`, `paint_dab(x, y, pressure)`, `end_paint() -> bool`, `cancel_paint()`; pre-stroke base + scratch re-composite; history capture at `end`; dirty marking
- [ ] 3.2 Bridge: find the topmost raster layer; refuse gracefully when there is none
- [ ] 3.3 `tools.{h,cpp}`: add `ToolId::Brush` and `ToolId::Pencil`, `B`/`Shift+B` cycling, per-tool cursors
- [ ] 3.4 `toolbox.{h,cpp}`: Brush/Pencil entries
- [ ] 3.5 `options_bar.{h,cpp}`: size, hardness, opacity, flow, mode combo, Auto Erase (Pencil); `[`/`]` and `Shift+[`/`Shift+]` shortcuts
- [ ] 3.6 `frame.{h,cpp}`: route paint pointer events to the bridge; rebuild on `changed`
- [ ] 3.7 Build green; M16–M20 self-tests still exit 0

## 4. Wave 3: self-test (sub-agent)

- [ ] 4.1 Extend `--self-test` (exit codes from 53): paint a scripted stroke and assert pixels change + dirty; opacity cap; flow ordering; Brush anti-aliased vs Pencil aliased; one history state per stroke; undo restores pixels
- [ ] 4.2 `xvfb-run` self-tests exit 0 (fixture and no-argument)

## 5. Close-out

- [ ] 5.1 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [ ] 5.2 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [ ] 5.3 Archive the change and commit
