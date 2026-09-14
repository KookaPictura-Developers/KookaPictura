# Agent guide — Kooka Pictura

Documentation-first project reimplementing **Adobe Photoshop CS6** (v13) in
**Rust + Qt6** on Linux. Specifications live in `docs/`; code lives in `crates/`.

## Layout

```
docs/            specification corpus (source of truth for behavior)
crates/
  pictura-core/     document/pixel types
  pictura-codec/     image codecs (PSD/PSB first)
  pictura-testkit/   verification harness (golden images, hashing)
  pictura-app/       Qt shell / cxx-qt bridge
```

## Commands

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo nextest run --workspace          # preferred when available
cargo deny check                       # licenses/advisories/bans
cmake -S . -B build && cmake --build build   # Qt app (M0+)
```

Toolchain is pinned by `rust-toolchain.toml` (1.98). Qt is the system Qt 6
(`qmake6 -query QT_VERSION`).

## Rules

1. **Specs are the contract.** Do not change files under `docs/` unless the task
   explicitly says so. Read the matching spec before implementing a behavior.
2. **Definition of done.** Non-trivial logic (a branch, loop, parser, or
   money/security path) ships with one runnable check: a unit test, a `demo()`
   self-check, or an integration test. Trivial one-liners need none.
3. **No unrequested abstractions.** No factory for one product, no trait with one
   implementation, no config for a constant. Delete over add.
4. **No new dependencies** without stating why in the commit/task result. Prefer
   std and already-present crates.
5. **Behavioral parity only where the oracle exists.** Adobe's closed algorithms
   are approximated and marked as such; never claim verified parity without a
   test that proves it.
6. **Determinism.** Golden comparisons run on CPU; GPU is an accelerator, not an
   oracle. Fix seeds for anything random.
7. **No comments explaining what the code says.** Comment only non-obvious
   intent. Mark deliberate shortcuts with a `ponytail:` comment naming the
   ceiling.
8. **Escalate, don't guess.** If a spec is ambiguous or a dependency/toolchain
   decision is needed, stop and report the blocker rather than inventing scope.

## Anti-patterns

- Weakening or deleting a test to make CI pass. Changing a golden baseline
  requires an explicit note in the task result.
- Committing build artifacts (`target/`, `build/`, `.opencode/node_modules/`).
- Touching unrelated files. Keep diffs scoped to the task.
