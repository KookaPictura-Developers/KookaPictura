# Tasks

## 1. The plane type

- [x] 1.1 Add `crates/pictura-core/src/plane.rs` with `Plane<T>`: `Deref<Target = [T]>`,
  `DerefMut` through `Arc::make_mut`, `From<Vec<T>>`, `AsRef<[T]>`, `Default`,
  the derived `Clone`/`Debug`/`PartialEq`/`Eq`, and the cross-type
  `PartialEq<Vec<T>>` / `PartialEq<&[T]>` / `PartialEq<[T]>` plus their reverse
  directions, re-exported from `lib.rs`; verify `cargo nextest run -p pictura-core`
  passes with a unit test proving a clone shares storage, a write forks only the
  written plane, and the original keeps its bytes. **Done.** `crates/pictura-core/src/plane.rs`: `Plane<T>` over `Arc<[T]>` with `Deref`/`DerefMut` via `Arc::make_mut`, `From<Vec<T>>`, `AsRef`, `FromIterator`, `as_slice`/`as_mut_slice`/`truncate`/`clear`/`extend`/`extend_from_slice`, `IntoIterator for &Plane`, and `PartialEq` against `Plane`/`Vec`/`[T]`/`&[T]`/`[T; N]`. Four unit tests cover share-then-fork, in-place unique writes, the comparison operands, and `as_deref`.
## 2. Convert the plane-bearing fields

- [x] 2.1 Change `PixelBuffer::data` from `Vec<T>` to `Plane<T>` and fix every
  `E0308` the compiler names in `pictura-core`; verify
  `cargo check -p pictura-core --all-targets` reports no type errors. **Done.** `PixelBuffer::data` is `Plane<T>`; `cargo check -p pictura-core --all-targets` clean after `PixelBuffer::new` and `from_rgba` gained `.into()`.
- [x] 2.2 Change `Channel::data` and `LayerMask::data` to `Plane<u8>` and fix the
  remaining `pictura-core` sites; verify `cargo check -p pictura-core --all-targets`
  exits 0 and `cargo nextest run -p pictura-core` passes. **Done.** `Channel::data` and `LayerMask::data` are `Plane<u8>`; `cargo check -p pictura-core --all-targets` exits 0 and `cargo nextest run -p pictura-core` passes.
- [x] 2.3 Walk the workspace's `E0308` list and convert each site: struct
  literals gain `.into()`, and any `Vec`-only call on a plane (`truncate`,
  `clear`, `push`, `extend`, `as_mut_slice`) becomes its plane equivalent;
  verify `cargo check --workspace --all-targets` exits 0 with **no assertion
  weakened** — no `assert_eq!` may be deleted, relaxed or turned into a
  length-only check to make it compile. **Done.** `cargo check --workspace --all-targets` exits 0 across `core`, `codec`, `render`, `paint`, `select`, `adjust`, `filters`, `ops`, `app` (about 450 construction sites gained `.into()`, plus `.to_vec()`/`.as_ref()` where a `Vec` was genuinely wanted and two `Option::map(Into::into)`). No assertion was deleted, relaxed or reduced to a length check: the only `assert_eq!` edits add `.to_vec()` on the plane side so the same bytes are compared.
- [x] 2.4 Add a test that cloning a document shares its planes, that a write
  through the clone leaves the original byte-identical, and that undo still
  restores bit-identically; verify `cargo nextest run -p pictura-core -p pictura_app`
  passes, including `undo_display_from_snapshot_composite_equals_full_recomposite`. **Done.** `a_cloned_document_shares_its_planes_until_a_write_forks_them` in `crates/pictura-core/src/tests.rs` proves the clone shares the composite and every channel, that writing one plane forks only it, that the snapshot keeps the old bytes, and that an unwritten plane stays shared. `cargo nextest run --workspace`: **1810 passed, 13 skipped, 0 failed**, including `undo_display_from_snapshot_composite_equals_full_recomposite`.
- [x] 2.5 Retire the `ponytail: full-document clones` note in
  `crates/pictura-app/src/history.rs` and state the new contract on
  `Document::clone`; verify both comments say what is shared and what forks. **Done.** The `ponytail: full-document clones` note in `crates/pictura-app/src/history.rs` is replaced by the sharing contract, and the `Document` doc now says a clone is a refcount bump that forks only a written plane.
## 3. Measure and document

- [x] 3.1 Extend the ignored profile in
  `crates/pictura-app/src/cxxqt_object/tests_profiles.rs` to print
  `Document::clone` and `Stroke::begin_at` for a 1-layer and an 8-layer 4000²
  document; verify the release run is recorded in the task result with the
  pre-change baseline (81 ms / 360 ms from `canvas-view-spec.md` § 3.4) beside
  the new figures. **Done.** `stroke_split_profile_4000` now prints `clone` and a steady-state `steady` (four more dabs) beside `begin`/`raster`. Release, 4000 deg2: `Document::clone` **0.004 ms** (1 layer) / **0.008 ms** (8 layers) against the 81 ms / 360 ms baseline in `canvas-view-spec.md` section 3.4; stroke start **5.5 ms**; first dab 21.8-28.0 ms (it carries the one-time 64 MB plane fork); four further dabs 3.3 ms (diameter 64) / 20.2 ms (diameter 500).
- [x] 3.2 Update the `M36` note in `docs/dev/canvas-compositing-plan.md` and the
  measured table in `docs/dev/canvas-view-spec.md` § 3.4 with the new stroke
  start and history figures; verify the edits name the measured numbers and
  every commit carrying them includes `TASK-ALLOWS-DOCS`. **Done.** `docs/dev/canvas-view-spec.md` section 3.4 carries the new table and both explanations; `docs/dev/canvas-compositing-plan.md` M36 is retitled to "history region deltas (deferred)" and records what landed. Commits carrying them include `TASK-ALLOWS-DOCS`.
## 4. Verification

- [x] 4.1 Run `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run --workspace` and `cargo test --workspace --doc`; verify
  every command exits 0 with no new warnings or failures. **Done.** `cargo fmt --all` clean; `cargo clippy --workspace --all-targets -- -D warnings` exit 0 (after removing two `.into()` the automated pass had added to a tuple and a cast); `cargo nextest run --workspace` **1810 passed / 13 skipped / 0 failed**; `cargo test --workspace --doc` 0 failed.
- [x] 4.2 Run `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` and
  `openspec validate --all --strict`; verify both exit 0 and record the totals
  (`test-report` passed/failed, self-test summary, openspec items) in the task
  result. **Done.** `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` exit 0: `TOTAL 2321 passed / 14 skipped / 0 failed`, app self-test **511 passed**, file-size OK, guard OK. `openspec validate --all --strict`: **131 passed, 0 failed**.