# Tasks

## 1. Region-delta storage

- [x] 1.1 Replace the whole-snapshot `Entry` with a base plus per-state deltas:
  tracked-plane tile diff at 64×64, the state's selection, and its metadata.
  **Done.**
- [x] 1.2 Fall back to a full state when the tracked-plane count, stride, or
  height changes; promote state 1 to an anchor when the depth bound evicts the
  base. **Done.**
- [x] 1.3 Keep the public `History` API and the 20-state / 10-snapshot bounds
  unchanged. **Done.**

## 2. Materialization

- [x] 2.1 Apply before/after tiles and adopt state metadata incrementally;
  rebuild from the nearest full anchor when the chain is broken. **Done.**
- [x] 2.2 Mark the deliberate ceilings with `ponytail:` comments (uncompressed
  tile lists, geometry-change full fallback). **Done.**

## 3. Verification

- [x] 3.1 Add a test proving ten small edits on a large document retain only the
  changed tiles (not ten full states) and that every state's bytes are exact.
  **Done.**
- [x] 3.2 Add a test proving a paint commit followed by undo restores the exact
  prior layer and composite pixels, and redo restores the committed pixels.
  **Done.**
- [x] 3.3 Add a differential test that undo/redo/jump for every state equal a
  reference copy of each captured `Document`. **Done.**
- [x] 3.4 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo nextest run --workspace`, `TASK_ALLOWS_DOCS=1 bash
  scripts/verify-fast.sh`, the CMake build plus both self-tests, and `openspec
  validate --all --strict`. **Done.**
