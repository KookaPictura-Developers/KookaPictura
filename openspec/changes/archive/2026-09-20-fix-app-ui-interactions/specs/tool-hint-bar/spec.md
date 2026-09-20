# tool-hint-bar Specification

## Purpose

A context hint strip in the bottom status bar that shows the active tool's
keyboard modifiers as bordered keycaps with descriptions, so the available
modifiers (for example Add to selection and Subtract from selection) are visible
without opening help. The hints are sourced from the command registry and the
tool shortcut table rather than a second shortcut table.

## ADDED Requirements

### Requirement: Context tool hint bar in the status bar

The bottom status bar SHALL host a hint bar for the active tool instead of a flat
hint string. The bar SHALL render each entry as a bordered keycap with an
adjacent description, and SHALL update when the active tool changes. For a
selection tool the bar SHALL include `Shift` = Add to selection and `Alt` =
Subtract from selection, and the per-tool set SHALL be contextual to the active
tool. The bar SHALL fall back to the existing text hint when the active tool has
no keycap entry, so it never renders empty for a known tool.

#### Scenario: The bar shows the active tool's modifiers [lth_context]

- **WHEN** a selection tool becomes active
- **THEN** the hint bar shows a `Shift` keycap described as Add to selection and
  an `Alt` keycap described as Subtract from selection

#### Scenario: The bar follows the active tool [lth_follow]

- **WHEN** the active tool changes
- **THEN** the hint bar replaces its keycaps with the new tool's context

#### Scenario: A tool with no keycaps falls back [lth_fallback]

- **WHEN** the active tool has no keycap entry
- **THEN** the status bar shows the tool's text hint rather than an empty bar

### Requirement: Hint sources and pressed-key highlight

Each keycap's label SHALL be sourced from the command registry's action shortcut
(`CommandRegistry::action(id)->shortcut()`) for command modifiers and from the
existing tool-shortcut accessor (`toolShortcutKeys()`) for tool letters; the
system SHALL NOT introduce a second shortcut table. While a key that matches a
shown keycap is pressed, that keycap SHALL render in a highlighted state, and the
highlight SHALL clear when the key is released.

#### Scenario: A keycap uses the registered shortcut [lth_source]

- **WHEN** a command entry is shown in the hint bar
- **THEN** its keycap text comes from that command's registered shortcut

#### Scenario: A pressed key highlights its keycap [lth_highlight]

- **WHEN** a key shown as a keycap is pressed and held
- **THEN** that keycap renders highlighted and returns to normal on release
