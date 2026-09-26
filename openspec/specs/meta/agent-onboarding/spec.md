# meta/agent-onboarding Specification

## Purpose
Defines what `DEVELOPING.md` must document so that a new contributor — human or
agent — can install and use the OpenSpec and Serena toolchain the repository
assumes.

## Requirements

### Requirement: OpenSpec installation is documented

`DEVELOPING.md` SHALL document how to install the OpenSpec CLI, the minimum
supported version, and that `openspec update` regenerates the `.opencode/`
skills and commands, whose diff must be committed. The version documented SHALL
be the same pin the CI guards workflow installs.

#### Scenario: A contributor can install OpenSpec

- **WHEN** a new contributor reads the tooling prerequisites
- **THEN** they find the install command, the version floor, and the
  `openspec update` step

#### Scenario: Documented version matches CI

- **WHEN** the version in `DEVELOPING.md` and the pin in the guards workflow
  are compared
- **THEN** they are the same version

### Requirement: Serena setup is documented

`DEVELOPING.md` SHALL document Serena's requirements and setup: the `serena`
executable, the MCP server declared in `opencode.json`, the one-time
`serena project index` after a fresh clone, the committed `.serena/`
configuration and memories, the ignored `cache/`, `project.local.yml`, and
`compile_commands.json`, and that C++ cross-file navigation needs a CMake
configure to produce `compile_commands.json`.

#### Scenario: A contributor can index the project

- **WHEN** a new contributor follows the Serena section after cloning
- **THEN** they can index the project and get C++ cross-file navigation

### Requirement: The LLM-assisted workflow is documented

`DEVELOPING.md` SHALL describe how the agent tooling divides responsibilities:
root `AGENTS.md` for agent rules, `.opencode/` for OpenSpec skills and commands,
and `.serena/memories/` for durable project knowledge. It SHALL state that a
change under `docs/` requires `TASK-ALLOWS-DOCS`, including when an agent makes
it.

#### Scenario: A contributor finds where a convention lives

- **WHEN** a contributor or agent needs a rule, a command, or project knowledge
- **THEN** `DEVELOPING.md` points to the file that owns it
