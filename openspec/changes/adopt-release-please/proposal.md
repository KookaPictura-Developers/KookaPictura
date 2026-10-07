# Proposal

## Why

Releases are currently manual: there is no tag, no `CHANGELOG.md`, and no
automated version bump, so the packaging workflows planned in #192 (AppImage,
deb, snap, flatpak, DMG, Windows) have no trigger point and every release
requires hand-editing `Cargo.toml` and writing notes. Conventional Commits are
the repository convention (the commitlint enforcement tracked in #69), which is
exactly the input release-please needs.

## What Changes

- Add a `release-please` GitHub Actions workflow on `master` that opens and
  maintains a Release PR; merging it creates the tag, `CHANGELOG.md` entry,
  and GitHub Release.
- Add the manifest config (`release-please-config.json` +
  `.release-please-manifest.json`) bootstrapped at `0.0.0`, with `initial-version`
  set to `0.1.0`, so the first merged Release PR proposes **v0.1.0**.
- Keep versions in sync across the hybrid build. The `rust` strategy cannot read
  this workspace (a virtual root `Cargo.toml` plus `version.workspace = true`
  members; see `design.md`), so use `release-type: simple` with a `version.txt`
  version file, and generic `extra-files` annotations that bump `Cargo.toml` and
  the CMake project version (adding a `VERSION` field to `project()`, which has
  none today).
- Document the release flow (Release PR → merge → tag → packaging trigger) in
  `DEVELOPING.md`.

## Capabilities

### New Capabilities

- `meta/release-please`: The release-please automation contract — Release PR
  lifecycle on `master`, version source of truth bootstrapped at 0.0.0 with a
  first release of 0.1.0, synchronized version files (`version.txt`,
  `Cargo.toml`, `CMakeLists.txt`), changelog generation, and tag creation as the
  trigger point for packaging workflows.

### Modified Capabilities

(none)

## Impact

- New files: `.github/workflows/release-please.yml`,
  `release-please-config.json`, `.release-please-manifest.json`, `version.txt`,
  `CHANGELOG.md` (generated on first release).
- Edited: `CMakeLists.txt` (add `VERSION` + version-update marker),
  `Cargo.toml` (version-update marker), `DEVELOPING.md` (release process
  section), `scripts/guard.sh` (fail on a dropped version marker).
- `Cargo.toml` and `CMakeLists.txt` move from `0.0.0` → `0.1.0` on the first
  merged Release PR, automated thereafter. `Cargo.lock` is refreshed by `cargo`
  on the next build rather than by the Release PR.
- CI: the workflow needs `contents: write`, `pull-requests: write`,
  `issues: write`, and the repository/organization setting that lets
  `GITHUB_TOKEN` open a pull request (now enabled). A PAT is deferred until
  packaging workflows exist, because `GITHUB_TOKEN`-created events do not
  trigger other workflows.
- The first `0.1.0` changelog covers the project history, since there is no
  prior release.
- No engine crates, Qt code, or spec corpus behavior changes.
