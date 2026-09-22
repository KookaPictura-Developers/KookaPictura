# Agent guide — Kooka Pictura

Documentation-first reimplementation of **Adobe Photoshop CS6** (v13) in
**Rust + Qt6** on Linux. `docs/` is the long-form behavioral contract; code
lives in `crates/`.

## Start here

- `docs/dev/STATE.md` — resume anchor ("where things are"). Read it first;
  update it at the end of a milestone.
- `docs/dev/testing-conventions.md` — how every test layer is built and reports.
  The source of truth for test mechanics; don't restate it, follow it.

## Layout

```
docs/          spec corpus + docs/dev/ working notes (read-only unless tasked)
openspec/      change proposals (changes/) + capability specs (specs/)
crates/        10 engine crates + the app
  core|codec|color|adjust|filters|ops|paint|select   spec math, no Qt
  pictura-render/   CPU compositing + GPU (wgpu/Vulkan) compute
  pictura-testkit/  golden-image compare / hashing
  pictura-app/      the only Qt crate — Rust bridge (src/cxxqt_object/*) + C++/Qt (cpp/)
scripts/       verification gates + Python oracle tools
```

The app is a Rust `staticlib` (`pictura_app`, built by Corrosion from Cargo)
linked into the C++ `pictura` binary. cxx-qt codegen is driven by
`crates/pictura-app/build.rs` + `src/cxxqt_object.rs`. **New `.cpp`/`.h` files
must be added to `CMakeLists.txt` explicitly — there is no globbing.**

## Commands

```bash
cargo fmt --all                                  # CI runs --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace                    # one crate/test: -p <crate> [name]
cargo test --workspace --doc                     # doctests; nextest skips them
cargo test -p pictura-render --test document_oracle       # one integration suite
cargo test -p pictura-core -- <name> --ignored --nocapture # ignored profiling/GPU
bash scripts/test-report.sh                      # nextest + doctests + app self-test -> unified report
bash scripts/verify-fast.sh                      # fmt, clippy, test-report, file-size, guard, openspec
bash scripts/verify-full.sh                      # CMake build first, then verify-fast
openspec validate --all --strict                 # openspec 1.3.1
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld && cmake --build build --parallel
./build/pictura --headless --self-test [file.psd]
```

Toolchain is pinned by `rust-toolchain.toml` (1.98). Qt is the system Qt 6
(`qmake6 -query QT_VERSION`). Cargo links with `lld` (`.cargo/config.toml`);
CMake builds with Ninja. `cargo deny check` is a no-op locally (no `deny.toml`);
CI runs it only when the file exists.

## Serena (symbol intelligence)

Serena is the project's symbol-level IDE (configured globally; see also
`.opencode/`). The repo is already a Serena project: `.serena/project.yml` (config)
and `.serena/memories/*` (project knowledge) belong with the repo — commit them
alongside it; only Serena's `cache/` and `project.local.yml` are gitignored.
Prefer its LSP tools (`find_symbol`, `find_referencing_symbols`,
`replace_symbol_body`, `rename_symbol`) over text search for reading and editing
code, and read the relevant memory before a non-trivial task.

- **Index upkeep.** `serena project index` is needed only once, after a fresh
  clone. During normal use Serena updates the index itself whenever files change,
  so edits and builds need no manual re-index. (A heavy Rust rebuild can still
  make rust-analyzer re-index and feel slow for a moment.)
- **C++ needs a compile database.** clangd, and therefore Serena's C++ cross-file
  references, reads `compile_commands.json` from the repo root. The CMake
  configure (command above) links it there from `build/`; if C++ navigation looks
  fuzzy, (re)configure first.
- **Keep memories current.** Update `.serena/memories/*` (via `write_memory`) when
  conventions or architecture change — they are the durable project knowledge.

## Headless mode

`./build/pictura --headless` selects the offscreen QPA plugin before
`QApplication` and implies `--self-test` when no document is given, so it never
blocks. The first check asserts the platform is `offscreen`. Explicit
`QT_QPA_PLATFORM=offscreen` and `xvfb-run` remain valid. `--interop-probe`
requires a real platform Vulkan instance and is **not** offscreen-compatible.

## Verification (the non-obvious parts)

- **The C++ self-test is a hand-rolled oracle**, not Qt Test/CTest:
  `crates/pictura-app/cpp/selftest*.cpp` + `selftest_report.*`, one long
  sequential `runSelfTest()`.
- It emits one token per check to stderr:
  `pictura self-test: PASS|SKIP|FAIL <suite> <name> ...`, closed by
  `SUMMARY passed=<n> failed=<n> skipped=<n>`. **The exit code is the failure
  identity** — `ST_FAIL(code)` returns its code, and codes are append-only.
  Names carry no milestone.
- Add a check with `ST_BEGIN` / `ST_PASS` / `ST_SKIP` / `ST_FAIL` / `ST_FINISH`,
  take the next free code, and keep each `selftest*.cpp` inside its
  `scripts/file-size-allowlist.txt` ceiling.
- Rust tests are std `#[test]` only (no framework). Oracle tests self-skip when
  `magick` / `psd-tools` are absent; CI's `oracles` job installs both. Profiling
  and GPU tests are `#[ignore]`d with a reason string (`--ignored --nocapture`).

## Spec workflow (OpenSpec)

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec) to turn work
into reviewable change proposals before code. `docs/` remains the long-form
contract; OpenSpec carries the per-change *requirements* and their task list.

- A change lives in `openspec/changes/<kebab-name>/`:
  `proposal.md` (why / what / capabilities), `design.md` (how),
  `specs/<capability>/spec.md` (ADDED / MODIFIED / REMOVED requirement deltas),
  `tasks.md` (implementation checklist).
- Capabilities are kebab-case names; each becomes `openspec/specs/<capability>/spec.md`
  once the change is archived. Prefer new capability names over `MODIFIED` unless the
  requirement itself changes.
- Commands: `/opsx-explore`, `/opsx-propose`, `/opsx-apply`, `/opsx-archive`
  (skills in `.opencode/skills/openspec-*`). Artifacts are generated from
  `openspec instructions <artifact> --change <name> --json`; validate before
  committing with `openspec validate --all --strict`.
- Format is strict: `### Requirement:` then `#### Scenario:` (exactly four `#`),
  normative SHALL/MUST wording, at least one scenario per requirement.
- Archive a completed change with `openspec archive <name>` to merge its deltas
  into `openspec/specs/`.
- The M0–M5 work predates this workflow; it is documented retroactively as the
  `openspec/changes/m0-*` … `m5-*` proposals.

## Rules

1. **Specs are the contract.** Do not change `docs/` unless the task says so.
   Any `docs/` change must carry `TASK-ALLOWS-DOCS` in the commit message (or
   run with `TASK_ALLOWS_DOCS=1`), or `scripts/guard.sh` fails. Commit docs
   separately; `GUARD_BASE=<ref>` re-checks committed docs changes.
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
9. **File size.** Target under **800 LOC**; hard cap **1200** for code and
   **1400** for tests. Tests are detected by path: Rust under `tests/` or named
   `tests.rs`, C++ `*_test.{cpp,h}`. A file over its cap must be listed in
   `scripts/file-size-allowlist.txt`, a ceiling that only
   shrinks. Split along class/concern seams with pure moves (no behavior change):
   C++ classes may span several `.cpp` translation units, Rust modules become
   submodule directories.
10. **Milestones (`mNN`) live only in comments, docs, and specs** — never in
    identifiers, string literals, or test names. `scripts/check-milestone-names.py`
    (run by `guard.sh`) enforces this.

## Anti-patterns

- Weakening or deleting a test to make CI pass. Changing a golden baseline
  requires an explicit note in the task result.
- Committing build artifacts (`target/`, `build/`, `.opencode/node_modules/`).
- Touching unrelated files. Keep diffs scoped to the task.
