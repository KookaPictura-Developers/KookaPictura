# Tasks

## 1. Strict-green baseline

- [x] 1.1 Rewrite the `## Purpose` of all 110 main specs from the archive placeholder to one or two sentences describing the capability. Verify: `grep -rl "TBD - created by archiving change" openspec/specs/ | wc -l` prints `0`.
- [x] 1.2 Bump the OpenSpec pin in `.github/workflows/guards.yml` from `1.3.1` to `1.13.2`. Verify: `grep openspec .github/workflows/guards.yml` names the pin.
- [x] 1.3 Run `openspec validate --all --strict` and confirm it exits `0` (was 110 failures).

## 2. Taxonomy config

- [x] 2.1 Add the `context` string (nine domains + the `{domain}/{capability}` rule) and the `rules` map (`proposal`, `specs` naming the path and the mirrored delta path) to `openspec/config.yaml`. Verify: `openspec instructions proposal --change openspec-tree-and-onboarding --json` still emits the project context.
- [x] 2.2 Track `openspec/config.yaml`. Verify: `git ls-files openspec/config.yaml` prints the path.

## 3. Move the corpus into the tree

- [x] 3.1 `git mv` each capability folder into its domain per the design D1 table (`document`, `codec`, `color`, `compositing`, `imaging`, `tools`, `ui`, `interop`, `verification`). Verify: `find openspec/specs -mindepth 2 -maxdepth 2 -type d | wc -l` prints `110` and no `spec.md` remains at one level (`ls openspec/specs/*/spec.md` finds none).
- [x] 3.2 Verify discovery by id: `openspec list --specs` reports two-segment ids and `openspec show compositing/layer-compositing` resolves. 
- [x] 3.3 Re-run `openspec validate --all --strict`; confirm it stays green after the move.

## 4. Update hand-written path references

- [x] 4.1 Rewrite the 99 flat refs in `docs/GLOSSARY.md`. Verify: `grep -c "openspec/specs/[a-z0-9-]*/spec.md" docs/GLOSSARY.md` prints `0` (nested `openspec/specs/<domain>/<cap>/spec.md` forms remain).
- [x] 4.2 Update the OpenSpec section of root `AGENTS.md` to the `{domain}/{capability}` layout, the mirrored delta path, and the pinned version.
- [x] 4.3 Update `docs/dev/STATE.md` — OpenSpec version `1.13.2`, the nested tree, 110 specs.
- [x] 4.4 Update `.serena/memories/` (`core.md`, `task_completion.md`, `suggested_commands.md`) flat paths to the tree. Verify: `grep -rn "openspec/specs/<flat>" .serena/memories/` finds none.
- [x] 4.5 Commit the `docs/` changes with `TASK-ALLOWS-DOCS` (guard rule 4). Verify: `bash scripts/guard.sh` passes.

## 5. Document onboarding in DEVELOPING.md

- [x] 5.1 Add OpenSpec to Prerequisites: install command, the `1.13.2` pin and `≥1.7.0` floor, and that `openspec update` regenerates `.opencode/` skills/commands that must be committed. Verify: the commands run as written; the version matches `guards.yml`.
- [x] 5.2 Add a Serena setup section: `serena` executable, the MCP entry in `opencode.json`, one-time `serena project index`, committed `.serena/` vs ignored `cache/`/`project.local.yml`/`compile_commands.json`, and the CMake configure for C++ navigation.
- [x] 5.3 Add an LLM-assisted-development section: how `AGENTS.md`, `.opencode/`, and `.serena/memories/` divide the work, and that a `docs/` change needs `TASK-ALLOWS-DOCS`. Verify: each named file exists.

## 6. Commit the regenerated tooling

- [x] 6.1 Review and commit the `.opencode/skills/openspec-*` and `.opencode/commands/opsx-*` diff, including the new `openspec-sync-specs` / `opsx-sync`. Verify: `grep generatedBy .opencode/skills/*/SKILL.md` shows `1.13.2` and `git status .opencode` is clean.

## 7. Integration

- [x] 7.1 Run `bash scripts/verify-fast.sh` end to end and confirm it passes (its `--strict` steps now green).
- [x] 7.2 Smoke the corpus: `openspec list --specs` shows 110 nested ids, `openspec show codec/psd-codec` resolves, `openspec validate --all --strict` exits `0`.
- [x] 7.3 Confirm the branch's commits use the conventional format with the issue number and `TASK-ALLOWS-DOCS` on docs commits.
