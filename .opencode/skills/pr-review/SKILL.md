---
name: pr-review
description: Review a GitHub pull request from a URL or PR number, check security, consistency, roadblocks, documentation, and code quality, and post findings as PR comments. Use when the user says "review PR", "review pull request", "code review", or gives a PR link or number to review.
allowed-tools: Bash(gh:*), Bash(git:*), Bash(cargo:*), Bash(rg:*)
license: MIT
compatibility: Requires authenticated gh CLI; run from the repository root.
metadata:
  author: KookaPictura
  version: "1.0"
---

# Pull request review

Review the code of one PR and post findings as comments. Input is a PR URL or
number (e.g. `https://github.com/KookaStudio/KookaPictura/pull/123`, `#123`,
`123`). If no PR is given, infer from conversation context; if ambiguous, ask.

## 1. Gather context

```bash
gh pr view <pr> --json number,title,body,author,url,state,isDraft,mergeable,mergeStateStatus,baseRefName,headRefName,headRefOid,files,additions,deletions,commits,reviews,comments,labels
gh pr diff <pr>
```

- Accept the URL directly (`gh pr view <url>` works); pass `-R owner/repo` when
  the PR is not in the current repo.
- Read the full changed files, not just hunks. Fetch the head and read blobs
  without touching the working tree:
  ```bash
  git fetch origin pull/<n>/head
  git show FETCH_HEAD:<path>
  ```
- To run tests against the PR, use a scratch worktree — never check out over
  the user's working tree:
  ```bash
  git worktree add /tmp/opencode/pr-<n> FETCH_HEAD
  ```
  Remove it with `git worktree remove /tmp/opencode/pr-<n>` when done.

## 2. Review dimensions

Check every dimension; skip none silently.

**Security**
- Secrets, tokens, or credentials in code, tests, or CI; secrets logged.
- Untrusted input: this repo parses user-supplied files (PSD and image
  codecs) — check bounds, length fields, allocation caps, and integer
  overflow before indexing.
- `unsafe` Rust blocks and C++ pointer/lifetime/UB hazards; Qt signal
  lifetime issues.
- Injection: command execution, path traversal, shell interpolation in CI
  workflows (`pull_request_target`, `${{ }}` in `run:`), unsafe deserialization.
- New dependencies: justify against `deny.toml` and AGENTS.md rule 4.
- Auth/permission changes, weak crypto, predictable randomness.

**Consistency**
- AGENTS.md rules: no explanatory comments, no unrequested abstractions, no
  new deps without justification, milestones (`mNN`) only in comments/docs/specs,
  file-size caps (800 target / 1200 code / 1400 tests, allowlist only shrinks).
- Reuses existing crates, helpers, and patterns instead of new ones.
- Naming, error handling, and module layout match neighboring code.
- Commits follow Conventional Commits with `(#issue)`; PR title carries the
  conventional prefix and issue number.

**Convention gates** — the repo's CI checks are objective; run them first:

```bash
gh pr checks <n>
```

- `Conventional PR title` (`amannn/action-semantic-pull-request`): the title
  must start with a listed type (`feat fix docs style refactor perf test build
  ci chore revert`). Squash-only merge makes the title the release commit, so
  it also carries the `(#issue)` suffix. Example:
  `feat: port the healing family (#10)`.
- `Conventional commits` (`commitlint.config.mjs`): every commit needs a
  non-empty type and subject, header ≤ 150 chars, no trailing period, and ends
  with `(#issue)`. Example: `fix(codec): RLE row padding (#12)`.
- `guards`: `docs/` changes need `TASK-ALLOWS-DOCS` in the commit, plus
  file-size caps, milestone-name, and OpenSpec checks.

Report each failing gate as a `[blocker]` finding with the gate name and its
exact error (e.g. `subject may not be empty [subject-empty]`). Never rewrite
commits, force-push, or retitle the PR to fix gates unless the user explicitly
asks; suggest the fix instead (`gh pr edit --title`, reword via rebase).

**Roadblocks** (must fix before merge)
- CI failing, merge conflicts, or `mergeStateStatus` blocked.
- Non-trivial logic without a runnable check (unit test, `demo()`, or
  integration test); GUI checks must be Qt Test under `cpp/tests/`, not new
  self-test codes.
- `docs/` changed without `TASK-ALLOWS-DOCS` in the commit.
- OpenSpec deltas fail `openspec validate --all --strict`.
- Golden baselines changed without justification; tests weakened or deleted.
- Behavior claims that contradict `docs/` or `openspec/specs/`.

**Documentation verification**
- PR description matches what the code actually does.
- User-visible behavior changes update the relevant `docs/` or OpenSpec
  specs; new capabilities have spec deltas; changed requirements use MODIFIED.
- Public APIs, commands, and flags are documented; no stale comments.

**Code quality**
- Correctness and edge cases (empty input, overflow, None/error paths).
- Error handling: no panics/unwraps on fallible input; errors propagate.
- Dead code, duplication, scope creep beyond the issue.
- Performance on hot paths; determinism (fixed seeds, CPU goldens).

**Verification** — run what is cheap and relevant, e.g.
`cargo clippy -p <crate> --all-targets -- -D warnings` and
`cargo nextest run -p <crate>`. If the user wants the full gate:
`bash scripts/verify-fast.sh`.

## 3. Draft findings

- Anchor each finding to a changed line: `path:line`, severity tag, and why.
- Severities: `[blocker]`, `[security]`, `[consistency]`, `[docs]`,
  `[quality]`, `[nit]`.
- No speculation: verify the finding against the head commit. If unsure, say
  so in the comment instead of asserting.
- Do not repeat findings already raised in `reviews`/`comments`; check them
  first.

## 4. Post comments

Inline comment on a changed line:

```bash
gh api "repos/{owner}/{repo}/pulls/<n>/comments" \
  -f body="**[security]** unchecked length at this line can OOM; validate against remaining bytes" \
  -f path="crates/codec/src/psd/reader.rs" \
  -f commit_id="<headRefOid>" \
  -F line=88 -f side=RIGHT
```

- `line` must be a line in the diff on the given `side` (`RIGHT` = new file).
  If the API rejects it, fold the finding into the review body instead.
- Aggregate anything not line-anchorable (summary, roadblocks, missing docs)
  into one review body:
  ```bash
  gh pr review <n> --comment --body-file /tmp/opencode/pr-<n>-review.md
  ```
- Post at most one review; if a review was already posted this session, reply
  in the existing thread or edit rather than duplicating.

## 5. Report

Print a summary to the user: verdict (comments posted), counts per severity,
the blocker list, and the URL of the posted review. State explicitly when no
issues were found.

## Guardrails

- Comment only. Never approve, request changes, merge, close, push, or edit
  the PR unless the user explicitly asks.
- Never modify the PR branch or the working tree; use a scratch worktree.
- Never paste secrets or full credentials into a comment.
- If `gh` is unauthenticated or the PR does not exist, stop and say so.
- If the diff is huge, review the highest-risk files first and say what was
  not reviewed.
