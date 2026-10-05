## ADDED Requirements

### Requirement: Bundled Noto Sans faces

The system SHALL ship the Noto Sans static faces `Regular`, `Medium`,
`SemiBold`, and `Bold`, upright and italic, as TrueType files under
`assets/fonts/`, registered in the Qt resource `assets/pictura.qrc`. The set
SHALL be the Google Fonts static-instance source (`notofonts/noto-fonts`), the
SIL Open Font License 1.1 text SHALL be kept at `LICENSES/NotoSans-OFL.txt`,
and the source and version SHALL be recorded in `assets/PROVENANCE.md`.

#### Scenario: The faces are embedded [bfn_faces_embedded]

- **WHEN** the application is built
- **THEN** each `:/fonts/NotoSans-<Weight>[Italic].ttf` resource exists and
  resolves through `QFontDatabase::addApplicationFont`

#### Scenario: The license and provenance ship [bfn_license]

- **WHEN** the repository is inspected
- **THEN** `LICENSES/NotoSans-OFL.txt` contains the SIL Open Font License 1.1
  and `assets/PROVENANCE.md` records the Noto Sans source and license

### Requirement: Pinned application UI font

The system SHALL register the bundled faces after the `QApplication` exists and
SHALL install Noto Sans at a fixed 12-pixel size as the application UI font on
every OS. The chrome size derivations that run two pixels under the application
font SHALL read the pinned pixel size rather than a point size.

#### Scenario: The application font is the bundled family [bfn_app_font]

- **WHEN** the application starts
- **THEN** `QApplication::font().family()` is `Noto Sans` and
  `QApplication::font().pixelSize()` is `12`

#### Scenario: The chrome keeps its relative size [bfn_chrome_size]

- **WHEN** a tab label or the status footer derives its size from the
  application font
- **THEN** it is set two pixels under the pinned application pixel size

### Requirement: Weighted chrome resolves to a bundled face

The system SHALL resolve the requested weight for the application font family to
the corresponding bundled face — `Medium` for 500, `SemiBold` for 600, and
`Bold` for 700 — rather than snapping to the nearest face of a system fallback.
The tab labels SHALL be weighted Bold. The italic and semibold faces SHALL
resolve likewise.

#### Scenario: Medium resolves to Medium [bfn_medium]

- **WHEN** a `QFont` for family `Noto Sans` with weight `QFont::Medium` is
  inspected with `QFontInfo`
- **THEN** its resolved family is `Noto Sans` and its resolved weight is
  `QFont::Medium`

#### Scenario: SemiBold resolves for the tab chrome [bfn_semibold]

- **WHEN** a `QFont` for family `Noto Sans` with weight `QFont::DemiBold` is
  inspected with `QFontInfo`
- **THEN** its resolved family is `Noto Sans` and its resolved weight is
  `QFont::DemiBold`

#### Scenario: Bold resolves for the tab chrome [bfn_bold]

- **WHEN** a `QFont` for family `Noto Sans` with weight `QFont::Bold` is
  inspected with `QFontInfo`
- **THEN** its resolved family is `Noto Sans` and its resolved weight is
  `QFont::Bold`

#### Scenario: Both tab bars carry the bold chrome font [bfn_tab_bars]

- **WHEN** the document tab bar and a panel tab bar are inspected
- **THEN** each bar's font resolves to the bundled family at weight
  `QFont::Bold` and at the pinned application size minus two pixels

#### Scenario: The weighted faces exist [bfn_styles]

- **WHEN** the styles of the `Noto Sans` family are listed
- **THEN** `Regular`, `Medium`, `SemiBold`, and `Bold` are present

#### Scenario: Italic semibold resolves [bfn_italic]

- **WHEN** a `QFont` for family `Noto Sans` is set to `QFont::DemiBold` and
  italic
- **THEN** its `QFontInfo` resolves to `Noto Sans`, is italic, and weighs
  `QFont::DemiBold`
