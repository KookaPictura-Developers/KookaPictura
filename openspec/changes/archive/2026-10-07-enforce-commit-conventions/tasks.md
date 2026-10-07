# Tasks: enforce-commit-conventions

## 1. Workflow

- [x] 1.1 `.github/workflows/pr-conventions.yml`: `pull_request_target` on `opened`/`reopened`/`edited`/`synchronize`, `contents: read` + `pull-requests: read`.
- [x] 1.2 Title job: `amannn/action-semantic-pull-request` pinned to a commit SHA, with the repository type list.
- [x] 1.3 Commit job: `actions/checkout@v7` + `wagoid/commitlint-github-action` pinned to a commit SHA.

## 2. Config

- [x] 2.1 `commitlint.config.mjs`: `@commitlint/config-conventional` with an explicit `type-enum` matching `AGENTS.md`, and overrides for `subject-case` and the body/footer line-length rules (both reject the repo's own message style).

## 3. Documentation

- [x] 3.1 `AGENTS.md` "Enforcement" line updated to the CI check.

## 4. Verification

- [x] 4.1 `openspec validate --all --strict`.
- [x] 4.2 `bash scripts/guard.sh`.
- [x] 4.3 YAML parses; `actionlint` 1.7.7 reports no issues.
- [ ] 4.4 Set `RELEASE_PLEASE_TOKEN` (so the Release PR triggers this workflow), then make `Conventional PR title` a required status check on `master` (repository setting, not committed). The commit job stays advisory: squash-only means branch commits never reach `master`.
