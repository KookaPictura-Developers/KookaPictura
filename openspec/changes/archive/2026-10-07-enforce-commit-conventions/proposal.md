# Proposal

## Why

release-please derives the version bump and changelog from Conventional Commits
on `master`. The repository allows squash merges only, so the **PR title becomes
the commit release-please parses**; a malformed title is silently ignored — no
changelog entry, no version bump, the change ships untagged. Nothing enforces
the format today (`AGENTS.md`: "review-enforced for now"), so correctness rests
entirely on reviewers. Tracked in #215; the broader commitlint + local-hook work
is #69.

## What Changes

- Add a `pr-conventions` GitHub Actions workflow that validates every PR title
  against Conventional Commits with the repository type list, and validates
  every commit in the PR.
- Pin the allowed types in `commitlint.config.mjs` so the contract lives in the
  repository rather than in a dependency default.

## Capabilities

### New Capabilities

- `meta/commit-conventions`: the CI contract that PR titles and commits use the
  Conventional Commits format release-please consumes.

### Modified Capabilities

(none)

## Impact

- New files: `.github/workflows/pr-conventions.yml`, `commitlint.config.mjs`.
- Edited: `AGENTS.md` (the "Enforcement" line under Git conventions).
- Adds two checks (`Conventional PR title`, `Conventional commits`); branch
  protection must make the title check required for a hard gate (it guards the
  string release-please reads). The commit check stays advisory.
- Uses `pull_request_target` so fork PRs are validated; the workflow runs no PR
  code and only reads `contents`/`pull-requests`.
- No engine crates, Qt code, or runtime behavior changes.
