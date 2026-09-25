# free-transform Specification

## ADDED Requirements

### Requirement: Preset warp style mesh generation

`pictura_render` SHALL expose `pub enum WarpStyle` with the variants `None`,
`Custom`, `Arc`, `ArcLower`, `ArcUpper`, `Arch`, `Bulge`, `ShellLower`,
`ShellUpper`, `Flag`, `Wave`, `Fish`, `Rise`, `Fisheye`, `Inflate`, `Squeeze`,
and `Twist`, each carrying a CS6 display name, a stable id, and a `from_id`
constructor over a full ordered list. It SHALL expose
`style_mesh(style: WarpStyle, bend: f64, rotate_vertical: bool, w: i32, h: i32)
-> Option<WarpMesh>`. `style_mesh` SHALL return `None` for `None`, `Custom`, a
non-finite `bend`, or a non-positive `w`/`h`, and otherwise SHALL return the
preset control net in the warp box's local space with `bend` clamped to
`[-100, 100]`.

The 15 preset constructions SHALL match SethRobinson/Patchy's
`generate_style_warp_mesh` (commit
`7d14d1f6ede2dc8fb52c11eefcc7cc8783473711`, MIT) within `1e-9` at
`bend ∈ {-100, -50, 0, 50, 100}` and both orientations, and `bend == 0` SHALL
equal `identity_mesh` for that style's natural grid within `1e-9`.

#### Scenario: A preset at bend zero is the style identity grid

- **WHEN** `style_mesh` is called at `bend = 0` for any of the 15 presets and either orientation
- **THEN** the returned net has that construction's `cols`/`rows` and every point equals `identity_mesh(cols, rows, w, h)` within `1e-9`

#### Scenario: None and Custom produce no preset mesh

- **WHEN** `style_mesh` is called with `WarpStyle::None` or `WarpStyle::Custom`
- **THEN** it returns `None`

#### Scenario: Vertical orientation transposes the construction

- **WHEN** `style_mesh` is called with `rotate_vertical = true` for a non-Twist preset on a non-square box
- **THEN** the returned net has the horizontal net's `cols`/`rows` swapped and its coordinates transposed

#### Scenario: Twist ignores the orientation

- **WHEN** `style_mesh(WarpStyle::Twist, bend, ...)` is called with `rotate_vertical` false and true
- **THEN** both calls return the same control net

#### Scenario: Negative and upper styles mirror their counterparts

- **WHEN** `style_mesh(WarpStyle::Arc, -bend, false, ...)` or `style_mesh(WarpStyle::ArcUpper, bend, false, ...)` or `style_mesh(WarpStyle::ShellUpper, bend, false, ...)` is called
- **THEN** the result is the vertical mirror (`y -> h - y`, rows reversed) of the positive `Arc`, `ArcLower`, or `ShellLower` net respectively

### Requirement: Warp preset command and dialog

The application SHALL provide an `Edit > Transform > Warp` command (id
`edit.transform.warp`) that opens a Warp dialog collecting a Warp Style, a Bend
percent, X and Y distortion percents, and a vertical-orientation flag. The
dialog SHALL disable the Bend, X/Y, and orientation controls while the style is
`None` or `Custom`.

On accept, the `PictureView` bridge `apply_warp_preset(path, style, bend,
distort_x, distort_y, rotate_vertical)` SHALL build the style's mesh from the
target layer's rect, call `transform_layer_warp` with
`WarpParams { distort_h: distort_x, distort_v: distort_y }`, recomposite, and
record exactly one `"Warp"` history state when the engine returns true. It SHALL
return false without mutating the document and without recording history when
there is no document, the style id is unknown, `style_mesh` returns `None`
(`None`/`Custom`), or the engine refuses. The command SHALL be enabled only when
the active view has a document and the selected path passes
`layer_can_free_transform`.

#### Scenario: A named preset applies in one history state

- **WHEN** `apply_warp_preset` runs on a raster layer with the `warpArc` id at 50% bend and no distortion
- **THEN** it returns true, the layer rect changes, and exactly one `"Warp"` history state is appended

#### Scenario: None and unknown styles add no history

- **WHEN** `apply_warp_preset` is called with `warpNone`, `warpCustom`, or an unknown id
- **THEN** it returns false and the history count and document are unchanged

#### Scenario: The command requires a transformable target

- **WHEN** no layer is selected or the selected path cannot be free-transformed
- **THEN** `Edit > Transform > Warp` is disabled
