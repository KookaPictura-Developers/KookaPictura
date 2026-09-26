## 1. CI workflow (`ci.yml`)

- [x] 1.1 Add `concurrency: {group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true}`
- [x] 1.2 Add workflow-level `paths-ignore: [docs/**, openspec/**, '**/*.md', .serena/**]` to the `push` and `pull_request` triggers
- [x] 1.3 Scope `oracles` to `cargo test --workspace --exclude pictura_app` and drop the Qt action plus Ninja/Vulkan headers from its apt step; keep `lld` and `liblcms2-dev`
- [x] 1.4 Replace the `cargo install cargo-deny --locked` step with `taiki-e/install-action@cargo-deny`, still gated on `hashFiles('deny.toml')`
- [x] 1.5 Add `timeout-minutes` to every job
- [x] 1.6 Restrict `on.push` to `main` in `ci.yml` and `guards.yml` so branch pushes run once via the pull request event
- [x] 1.7 Split `ci.yml` into a `lint` job (fmt, clippy, cargo-deny) and a `test` job (nextest, doctests, JUnit) that runs the whole workspace
- [x] 1.8 Evaluate moving the app tests into the Qt headless job; reverted after measuring it made that job the critical path (see design D9)
- [x] 1.9 Bump `actions/checkout`, `actions/upload-artifact`, and `mozilla-actions/sccache-action` to their Node-24 releases

## 2. Guards and DCO workflow

- [x] 2.1 Add `.github/workflows/guards.yml` (no path filter) running `scripts/guard.sh`, `openspec validate --all --strict`, and `scripts/check-file-size.sh`, installing the pinned `@fission-ai/openspec@1.3.1`
- [x] 2.2 Move the DCO check out of `ci.yml` into the always-running workflow and extend it to `push`, using `github.event.before..github.event.after` on push and `base.sha..head.sha` on pull requests

## 3. Local verification loop

- [x] 3.1 Add the docs-only fast path to `scripts/verify-fast.sh`: compute changed paths from the merge base with `main` plus untracked files, and run only `guard.sh` + `openspec validate` when every path matches the ignored set; empty diff falls through to the full gate
- [x] 3.2 Make `scripts/verify-full.sh` pass `-DCMAKE_CXX_COMPILER_LAUNCHER=sccache` only when `command -v sccache` succeeds
- [x] 3.3 Document `RUSTC_WRAPPER=sccache`, `cargo sweep`/`cargo clean`, and the CMake launcher in `DEVELOPING.md`

## 4. Verification

- [x] 4.1 `openspec validate --all --strict` passes
- [x] 4.2 `bash scripts/verify-fast.sh` and `bash scripts/verify-full.sh` pass with the same result as before
- [x] 4.3 Confirm `cargo test --workspace --exclude pictura_app` needs no Qt (run it on a tree without Qt configured, or confirm from the build graph)
- [ ] 4.4 After merge, observe one docs-only push (guards only), one code push (all jobs), and two rapid pushes (first cancelled)
