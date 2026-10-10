---
name: issue-review
description: Review open GitHub issues older than 1 day, verify against the current codebase whether the bug is fixed or the feature already exists, and label stale issues "can be closed". Use when the user says "issue review", "review issues", "triage issues", or asks whether open issues are still valid.
allowed-tools: Bash(gh:*), Bash(git:*), Bash(jq:*), Bash(date:*), Bash(rg:*)
license: MIT
compatibility: Requires authenticated gh CLI; run from the repository root.
metadata:
  author: KookaPictura
  version: "1.0"
---

# Issue review

Triage open GitHub issues against the current code. Label an issue
`can be closed` only when code, tests, or commits prove the bug is fixed or the
requested feature already exists. Never close issues, never edit issue text,
and never touch issues younger than 24 hours.

## 1. List candidates

Open issues created before today, minus ones already reviewed:

```bash
gh issue list --state open --limit 200 \
  --search "created:<$(date +%Y-%m-%d)" \
  --json number,title,createdAt,labels,body |
jq --arg cutoff "$(date -u -d '24 hours ago' +%Y-%m-%dT%H:%M:%SZ)" \
  '[.[] | select(.createdAt < $cutoff)
        | select([.labels[].name] | index("can be closed") | not)
        | {number, title, createdAt, labels: [.labels[].name], body}]'
```

If the result is empty, report that and stop. With many candidates, batch the
per-issue verification with `explore` subagents.

## 2. Verify each issue against the code

Extract the concrete claim (bug, missing feature, chore) from the title and
body, then gather evidence.

**Fix commits — cheapest signal.** Every commit in this repo ends with its
issue number in parentheses, so:

```bash
git log --all --oneline --grep "(#<number>)"
```

Read any hit and confirm it actually implements or fixes the claim.

**Code search.** Grep for the symbols, menu items, error strings, file paths,
or behavior named in the issue across `crates/`, `docs/`, and
`openspec/specs/` (including `openspec/changes/archive/` — the issue may
describe spec'd behavior that already shipped). Prefer Serena LSP tools
(`find_symbol`, `find_referencing_symbols`) for Rust symbols.

**Discussion.** Run `gh issue view <number> --comments` for linked PRs,
"fixed by" notes, or scope changes that settle the issue.

**Reproduce, when cheap.** For bugs, run the referenced test
(`cargo nextest run -p <crate> <name>`) or the smallest reproduction. A
passing regression test is decisive; a failing one means the issue is valid.

## 3. Decide

| Evidence | Action |
| --- | --- |
| Code, test, or merged commit proves the bug is fixed or the feature exists | Add `can be closed` |
| Code still shows the bug or lacks the feature | Leave open |
| Ambiguous, needs a product decision, or external dependency | Leave open, flag for human review |

Never label on a hunch. "Couldn't find it quickly" is not evidence; name the
commit, `file:line`, or test that resolves the claim.

## 4. Label

```bash
gh issue edit <number> --add-label "can be closed"
```

The label already exists in this repo (color `#b1a8c4`). If it is missing,
create it once and retry:

```bash
gh label create "can be closed" \
  --description "Reviewed: appears fixed or no longer valid" --color b1a8c4
```

Add only this label. Never remove or alter other labels, and never post a
comment unless the user asks.

## 5. Report

Print one row per candidate: issue number, title, verdict (labeled / still
valid / needs review), and the evidence (commit hash, `file:line`, test name).
End with counts, and say explicitly when nothing was labeled.

## Guardrails

- Labeling is the only write; never close an issue or PR.
- Never label an issue created less than 24 hours ago, even if it looks stale.
- One verdict per issue: do not relabel, unlabel, or revisit issues already
  carrying `can be closed`.
- Leave issues that depend on another repo, external services, or a product
  decision to a human.
- If `gh` is unauthenticated or the repo has no remote, stop and say so.
