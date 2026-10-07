# Design

## Context

See proposal.md for motivation. The repository is a hybrid build: a Cargo
workspace at the root plus a CMake `project()`. Two facts from the current tree
drive the whole design.

1. The root `Cargo.toml` is a **virtual manifest** — it has `[workspace]` and
   `[workspace.package].version = "0.0.0"`, but no `[package]`. The eleven
   member crates inherit the version with `version.workspace = true` rather than
   repeating it.
2. The default branch is `master` (renamed in #206), and Conventional Commits
   are the repository convention (enforcement tracked in #69).

release-please's `rust` strategy is the obvious fit, but it cannot read this
layout. On a workspace it iterates members and then updates the root
`Cargo.toml` with its `CargoToml` updater, which throws when the root has no
`[package]` table (release-please #1998). The `cargo-workspace` plugin then
compounds it: it rejects `version.workspace = true` and demands a literal
`[package].version` in every member (release-please #2111, still open). Making
either work means restructuring the workspace — giving all eleven crates
literal versions and the virtual root a fake `[package]` — which is far larger
than the release automation itself.

## Goals / Non-Goals

**Goals:**

- Tagged releases driven by Conventional Commits, first release `v0.1.0`,
  bootstrapped from a `0.0.0` manifest.
- `version.txt`, `Cargo.toml`, and `CMakeLists.txt` updated to the same version
  in one Release PR.
- A tag/release event packaging workflows can key on.
- No change to the Cargo workspace layout or the CMake build.

**Non-Goals:**

- Per-crate versions. The workspace is released in lockstep; there is one
  version for the whole project.
- Packaging workflows themselves (#192's AppImage/deb/snap/etc.).
- A PAT. The default `GITHUB_TOKEN` is used until packaging workflows need to
  be triggered by the release event.

## Decisions

### Decision: `release-type: simple`, not `rust`

The `simple` strategy only reads and writes a `version.txt` and the changelog;
it does not parse `Cargo.toml` at all. Both version files are then updated by
the strategy-independent `extra-files` mechanism, which works on any layout.

Alternatives considered:

- **`release-type: rust`** — fails on the virtual workspace root (above).
- **`rust` + `cargo-workspace`** — fails on `version.workspace = true`; fixing
  the workspace to satisfy it is a large, unrelated refactor.
- **`release-type: go`** — also has no version file, so it would avoid
  `version.txt`, but labeling a Rust/Qt repo a Go release is a trap for the next
  maintainer. `simple` is the documented generic strategy and its name says what
  it is.

### Decision: `initial-version: "0.1.0"`

release-please takes the first release version from the strategy, not from the
commit types: when the manifest reads `0.0.0` it treats the project as
unreleased and calls `initialReleaseVersion()`, whose default is `1.0.0`
(`src/strategies/base.ts`; the pre-major options are also ignored at bootstrap,
release-please #2087). `simple` does not override it, so the first Release PR
would propose `1.0.0`. Setting `initial-version` to `0.1.0` pins it.

Consequence: the bootstrap version is fixed rather than derived from commit
types. A breaking commit before the first release would still yield `0.1.0`,
not `1.0.0` — release-please has no commit-derived bootstrap. The spec delta
records this actual behavior; the earlier "breaking → 1.0.0" scenario was based
on an incorrect assumption and is removed.

### Decision: generic `extra-files` annotations

`Cargo.toml` and `CMakeLists.txt` each carry an `x-release-please-version`
annotation on the line holding the version. The generic updater replaces the
semver-looking token on that line and leaves the rest of the file — comments,
formatting, the CMake `project()` signature — untouched. Using one mechanism for
both files keeps the config small.

### Decision: leave `Cargo.lock` alone

Only the `rust` strategy updates `Cargo.lock`, and the `simple` updaters cannot
target the workspace-member entries: the `toml` updater filters on JSONPath but
runs against a parser whose scalars are position-tagged objects, so a
`@.name.startsWith(...)` filter cannot evaluate. Rather than list eleven JSONPath
entries with no stable surface, the release PR leaves `Cargo.lock` untouched;
the next `cargo` invocation rewrites the member versions. This means no tagged
release commit is `--locked`-buildable until something runs `cargo`: no CI job
uses `--locked`, but `scripts/third-party-licenses.py` runs
`cargo metadata --locked` and will need a plain cargo run first, and the #192
packaging workflows must not assume `--locked` at the tag. `ponytail:` if the
lockfile must be updated inside the Release PR, switch the root `Cargo.toml` to
a package manifest and use the `rust` strategy.

### Decision: `GITHUB_TOKEN`, with a PAT for packaging

The workflow runs with the built-in token. That is only enough to open the
Release PR if the repository (and the organization above it) allows it, via
*Settings → Actions → General → Workflow permissions → Allow GitHub Actions to
create and approve pull requests*; the repo-level setting was blocked by the
organization until that was enabled, and it now is. `GITHUB_TOKEN` still cannot
trigger *other* workflows from the PR or release (GitHub suppresses runs caused
by that token), which is the packaging trigger deferral the proposal calls out.
The token input is `secrets.RELEASE_PLEASE_TOKEN || secrets.GITHUB_TOKEN`, so
adding a PAT later is a secret, not a workflow edit. A PAT is required, not
optional, if required status checks are ever enabled on `master`, because the
default token's PR would then be unmergeable.

## Risks / Trade-offs

- **Release PR gets no CI checks** → GitHub does not run workflows for
  `GITHUB_TOKEN`-authored PRs, so a Release PR shows no status. Fine while
  `master` is unprotected; set the PAT before requiring status checks.
- **First changelog sweeps the project history** → with no prior release and no
  `bootstrap-sha`, the `0.1.0` entry lists everything release-please scans
  (capped at its commit-search depth), and the first run is the heaviest. This
  is intended for a first release; the workflow timeout is 20 minutes.
- **`version.txt` adds a version surface** → release-please keeps it in sync
  with Cargo/CMake, and the manifest is the real source. The trade is a
  readable, strategy-neutral version file.
- **`initial-version` pins the bootstrap** → if we ever want a different first
  release, edit the config before merging the first Release PR; afterwards the
  manifest drives everything.
- **`extra-files` needs the annotation to survive** → a future edit that drops
  the `# x-release-please-version` comment silences the bump for that file
  without error. `scripts/guard.sh` fails when either marker is missing.

## Migration Plan

1. Land config, manifest, `version.txt`, annotations, workflow, and docs.
2. On the first push to `master`, release-please opens the `0.1.0` Release PR.
3. Merging it sets `version.txt`, `Cargo.toml`, `CMakeLists.txt`, and the
   manifest to `0.1.0`, writes `CHANGELOG.md`, and tags `v0.1.0`.

Rollback: delete the workflow file; the config and annotations are inert
without it.
