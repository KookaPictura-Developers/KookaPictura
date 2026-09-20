## ADDED Requirements

### Requirement: Document tab label weight and padding

The document tab bar SHALL style its tab labels with a medium font weight
(`font-weight: 500`) and SHALL add a few pixels of right padding to each
`QTabBar#documentTabBar::tab`, so an active label does not touch the close
control or the next tab. The style rule SHALL be scoped to
`QTabBar#documentTabBar::tab` and SHALL NOT change the unscoped panel
`QTabBar::tab` rules or the panel tab bar. The change SHALL be a stylesheet
edit only: no tab-position, ordering, or close-control geometry changes, and the
self-test line that string-matches the theme SHALL NOT be edited.

#### Scenario: Active tab label is medium weight [ldt_tab_weight]

- **WHEN** a document tab is shown active
- **THEN** its label uses font weight 500 and the tab carries right padding

#### Scenario: Panel tabs are unaffected [ldt_tab_scope]

- **WHEN** a panel group tab is shown
- **THEN** its weight and padding are unchanged from before
