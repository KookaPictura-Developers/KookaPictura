# Spec Delta

## Purpose

Defines where capability specs live in the `openspec/` corpus, how a capability's
identifier derives from its directory path, how change deltas mirror that path,
and the quality bar every main spec meets.

## ADDED Requirements

### Requirement: Capability specs live in a domain tree

The spec corpus SHALL store each capability at
`openspec/specs/{domain}/{capability}/spec.md`, where `{domain}` and
`{capability}` are kebab-case path segments. A capability's identifier SHALL be
its directory path relative to `openspec/specs/`, using `/` as the separator
(for example `compositing/layer-compositing`). No capability spec SHALL live
directly under `openspec/specs/` or at a depth greater than two segments.

#### Scenario: Nested spec is discovered

- **WHEN** `openspec list --specs` runs against the corpus
- **THEN** every capability is reported with its two-segment relative path id

#### Scenario: A capability resolves by its path id

- **WHEN** `openspec show compositing/layer-compositing` runs
- **THEN** it resolves to `openspec/specs/compositing/layer-compositing/spec.md`

### Requirement: Domains are a declared, closed set

Every capability SHALL belong to exactly one domain, and the domain set SHALL be
declared in `openspec/config.yaml`. The corpus SHALL use the areas
`document`, `codec`, `color`, `compositing`, `imaging`, `tools`, `ui`,
`interop`, `verification`, and `meta` (repository and development-process
contracts). Introducing a new domain or moving a capability between domains
SHALL require a config and layout change, not an ad-hoc folder.

#### Scenario: Every capability has one declared domain

- **WHEN** the directories directly under `openspec/specs/` are listed
- **THEN** each is one of the declared domains, and every capability folder has
  exactly one parent domain

#### Scenario: The taxonomy is discoverable when proposing

- **WHEN** `openspec instructions proposal` runs for a new change
- **THEN** the project context names the domains and the `{domain}/{capability}`
  path rule

### Requirement: Change deltas mirror the main spec path

A capability delta in a change SHALL be written at
`openspec/changes/<name>/specs/{domain}/{capability}/spec.md`, mirroring the
main spec's path. Archiving SHALL merge the delta into the main spec at the same
relative path and SHALL NOT flatten it into a single segment.

#### Scenario: A nested delta archives to the nested main spec

- **WHEN** a change carrying a delta at `specs/{domain}/{capability}/spec.md` is archived
- **THEN** the merged main spec is written to
  `openspec/specs/{domain}/{capability}/spec.md`

### Requirement: A pinned CLI with nested discovery

The OpenSpec CLI used locally and in CI SHALL be a single pinned version no
lower than 1.7.0, the first release that discovers nested spec paths. The
documented version and the CI install SHALL name the same pin.

#### Scenario: CI installs a nested-capable version

- **WHEN** the guards workflow installs OpenSpec and validates the corpus
- **THEN** it installs the documented pin, which is ≥1.7.0, and validation
  discovers every nested spec

### Requirement: Every main spec states its purpose

Every main spec SHALL contain a Purpose section describing what the capability
is for. The archive placeholder `TBD - created by archiving change …` SHALL NOT
remain in any main spec, and `openspec validate --all --strict` SHALL pass.

#### Scenario: Strict validation passes

- **WHEN** `openspec validate --all --strict` runs against the corpus
- **THEN** every spec passes and the command exits zero

#### Scenario: A placeholder Purpose fails strict

- **WHEN** a main spec still contains the archive placeholder Purpose
- **THEN** `openspec validate --all --strict` reports it as a failure
