## MODIFIED Requirements

### Requirement: Context tool hint bar in the status bar

The bottom status bar SHALL host a hint bar for the active tool instead of a flat
hint string. The bar SHALL render each entry as a bordered keycap with an
adjacent description, and SHALL update when the active tool changes. For a
selection tool the bar SHALL include `Shift` = Add to selection and `Alt` =
Subtract from selection, and the per-tool set SHALL be contextual to the active
tool. The Move tool's nudge hint SHALL render as four separate directional
chevrons — up, down, left, and right — instead of a single `Arrows` keycap. The
bar SHALL fall back to the existing text hint when the active tool has no keycap
entry, so it never renders empty for a known tool.

#### Scenario: The bar shows the active tool's modifiers [lth_context]

- **WHEN** a selection tool becomes active
- **THEN** the hint bar shows a `Shift` keycap described as Add to selection and
  an `Alt` keycap described as Subtract from selection

#### Scenario: The bar follows the active tool [lth_follow]

- **WHEN** the active tool changes
- **THEN** the hint bar replaces its keycaps with the new tool's context

#### Scenario: The arrows hint is four chevrons [lth_arrows_chevrons]

- **WHEN** the Move tool is active
- **THEN** its nudge hint shows four separate up/down/left/right chevrons rather
  than a single `Arrows` keycap

#### Scenario: A tool with no keycaps falls back [lth_fallback]

- **WHEN** the active tool has no keycap entry
- **THEN** the status bar shows the tool's text hint rather than an empty bar
