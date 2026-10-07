# Design

## Why the PR title is the primary gate

The repository disables merge and rebase commits (squash only), so the only
message release-please ever sees on `master` is the squash commit, which
GitHub's "Default to PR title for squash merge commits" setting fills from the
PR title. Individual branch commits never reach `master`, which makes the title
the high-leverage check and the commit check an advisory hygiene gate.

## Why `pull_request_target`

The repository is public, so it accepts fork pull requests, and the
title action documents `pull_request_target` as the trigger for fork-based
workflows (a plain `pull_request` from a fork has no token it accepts). Neither
job checks out or executes PR-authored code: the title job reads the event
payload, and the commit job checks out the base ref and reads the PR's commits
through the API. The workflow requests only `contents: read` and
`pull-requests: read`, and disables persisted checkout credentials.

## Why two actions

- `amannn/action-semantic-pull-request@v6` validates the title against the
  Conventional Commits header with a configurable type list, no repository
  toolchain.
- `wagoid/commitlint-github-action@v6` lints the PR's commits with `commitlint`
  and uses the in-repo `commitlint.config.mjs`. It needs no `package.json`: the
  action supplies commitlint, and the config `extends` the bundled
  `@commitlint/config-conventional`.

## commitlint overrides

The config extends `@commitlint/config-conventional` but disables two inherited
rules that contradict this repository's documented style: `subject-case`
(config-conventional rejects every uppercase-first subject, e.g. `RLE row
padding`) and the body/footer line-length limits (long `Source: <url>` footers
are legitimate). `header-max-length` is raised to 150.

## Known limits

- **The workflow cannot run on the PR that introduces it.** GitHub uses the
  base branch's workflow file for `pull_request_target`, and the file is not on
  `master` yet. The workflow must be merged before the checks can be made
  required.
- **The config is read from the base branch**, so a PR that edits
  `commitlint.config.mjs` is validated against the previous config (one-PR lag).
  Do not "fix" this by checking out the PR head: it would execute PR-authored
  `.mjs` under the base repository's token.
- **GitHub's revert button produces a `Revert "..."` title**, which fails the
  title check. Retitle such a PR `revert: ...`; the `revert` type is allowed.
- **The type list is duplicated** in `pr-conventions.yml` (consumed by the
  title action) and `commitlint.config.mjs`. The title action cannot read the
  commitlint config, so both lists must be edited together.
- **Making the checks required is a repository setting**, not a committed file
  (`tasks.md` 4.4). Until it is set, the checks only report.
- **A required title check can stall the Release PR.** The release-please
  workflow opens the Release PR with `secrets.RELEASE_PLEASE_TOKEN ||
  secrets.GITHUB_TOKEN`; under the plain `GITHUB_TOKEN`, GitHub does not start
  workflows for events that token creates, so this workflow never runs on the
  Release PR and a required check would stay "Expected" forever. Set
  `RELEASE_PLEASE_TOKEN` before making the title check required.

## Not in scope

The `(#N)` issue-number suffix and `TASK-ALLOWS-DOCS` marker are repository
conventions beyond what release-please needs; enforcing them is issue #69.
