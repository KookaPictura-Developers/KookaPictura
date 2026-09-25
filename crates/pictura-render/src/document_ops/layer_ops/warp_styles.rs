//! The CS6 `Warp Style` presets: the 15 named control-net constructions that
//! feed [`super::warp::transform_layer_warp`].
//!
//! Ported from SethRobinson/Patchy's `generate_style_warp_mesh`
//! (`src/core/warp_mesh.cpp`, commit `7d14d1f6ede2dc8fb52c11eefcc7cc8783473711`,
//! MIT), which publicly documented Photoshop's own bakes to ~2.4e-6 px against
//! COM captures. The constructions are math, not Patchy code.
//!
//! `style_mesh` returns `None` for [`WarpStyle::None`] and
//! [`WarpStyle::Custom`]: neither is a preset construction (a custom net is
//! built by the interactive tool, which is not shipped). A preset at `bend == 0`
//! is the identity mesh of that style's natural grid.
//!
//! ponytail: Patchy's captures are Photoshop 2026; CS6 preset equivalence is
//! assumed (all 15 names match). The golden table in `warp_styles_tests.rs` is
//! the oracle, not a claimed CS6 pixel match.

use std::f64::consts::PI;

use super::warp::{identity_mesh, WarpMesh};

/// The CS6 `Warp Style` pop-up in display order. `None` and `Custom` are not
/// preset constructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpStyle {
    None,
    Custom,
    Arc,
    ArcLower,
    ArcUpper,
    Arch,
    Bulge,
    ShellLower,
    ShellUpper,
    Flag,
    Wave,
    Fish,
    Rise,
    Fisheye,
    Inflate,
    Squeeze,
    Twist,
}

impl WarpStyle {
    /// Every style in CS6 pop-up order.
    pub const ALL: [WarpStyle; 17] = [
        WarpStyle::None,
        WarpStyle::Custom,
        WarpStyle::Arc,
        WarpStyle::ArcLower,
        WarpStyle::ArcUpper,
        WarpStyle::Arch,
        WarpStyle::Bulge,
        WarpStyle::ShellLower,
        WarpStyle::ShellUpper,
        WarpStyle::Flag,
        WarpStyle::Wave,
        WarpStyle::Fish,
        WarpStyle::Rise,
        WarpStyle::Fisheye,
        WarpStyle::Inflate,
        WarpStyle::Squeeze,
        WarpStyle::Twist,
    ];

    /// The stable id the app combo passes to [`WarpStyle::from_id`].
    pub fn id(self) -> &'static str {
        match self {
            WarpStyle::None => "warpNone",
            WarpStyle::Custom => "warpCustom",
            WarpStyle::Arc => "warpArc",
            WarpStyle::ArcLower => "warpArcLower",
            WarpStyle::ArcUpper => "warpArcUpper",
            WarpStyle::Arch => "warpArch",
            WarpStyle::Bulge => "warpBulge",
            WarpStyle::ShellLower => "warpShellLower",
            WarpStyle::ShellUpper => "warpShellUpper",
            WarpStyle::Flag => "warpFlag",
            WarpStyle::Wave => "warpWave",
            WarpStyle::Fish => "warpFish",
            WarpStyle::Rise => "warpRise",
            WarpStyle::Fisheye => "warpFisheye",
            WarpStyle::Inflate => "warpInflate",
            WarpStyle::Squeeze => "warpSqueeze",
            WarpStyle::Twist => "warpTwist",
        }
    }

    /// The CS6 display name.
    pub fn display_name(self) -> &'static str {
        match self {
            WarpStyle::None => "None",
            WarpStyle::Custom => "Custom",
            WarpStyle::Arc => "Arc",
            WarpStyle::ArcLower => "Arc Lower",
            WarpStyle::ArcUpper => "Arc Upper",
            WarpStyle::Arch => "Arch",
            WarpStyle::Bulge => "Bulge",
            WarpStyle::ShellLower => "Shell Lower",
            WarpStyle::ShellUpper => "Shell Upper",
            WarpStyle::Flag => "Flag",
            WarpStyle::Wave => "Wave",
            WarpStyle::Fish => "Fish",
            WarpStyle::Rise => "Rise",
            WarpStyle::Fisheye => "Fisheye",
            WarpStyle::Inflate => "Inflate",
            WarpStyle::Squeeze => "Squeeze",
            WarpStyle::Twist => "Twist",
        }
    }

    /// Parse a stable id; `None` for an unknown string.
    pub fn from_id(id: &str) -> Option<WarpStyle> {
        WarpStyle::ALL.into_iter().find(|s| s.id() == id)
    }

    /// Whether the style names a preset construction (not `None`/`Custom`).
    pub fn is_preset(self) -> bool {
        !matches!(self, WarpStyle::None | WarpStyle::Custom)
    }
}

/// Append a 4-point straight row spanning `[0, width]` at `y`.
fn push_identity_row(mesh: &mut WarpMesh, width: f64, y: f64) {
    mesh.points.push((0.0, y));
    mesh.points.push((width / 3.0, y));
    mesh.points.push((2.0 * width / 3.0, y));
    mesh.points.push((width, y));
}

/// Append one row of the classic cubic circle-arc approximation: the arc with
/// horizontal chord `ax..bx` at `y`, central angle `2·theta`, bulging toward `-y`
/// when `bulge_up` (content space has +y down).
fn push_arc_row(mesh: &mut WarpMesh, ax: f64, bx: f64, y: f64, theta: f64, bulge_up: bool) {
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();
    let radius = (bx - ax) / (2.0 * sin_theta);
    let handle = (4.0 / 3.0) * (theta / 2.0).tan() * radius;
    let dy = if bulge_up {
        -handle * sin_theta
    } else {
        handle * sin_theta
    };
    mesh.points.push((ax, y));
    mesh.points.push((ax + handle * cos_theta, y + dy));
    mesh.points.push((bx - handle * cos_theta, y + dy));
    mesh.points.push((bx, y));
}

/// Mirror a mesh vertically: `y -> height - y`, rows reversed.
fn mirror_vertically(mesh: &WarpMesh, height: f64) -> WarpMesh {
    let mut out = WarpMesh {
        rows: mesh.rows,
        cols: mesh.cols,
        points: Vec::with_capacity(mesh.points.len()),
    };
    for row in 0..mesh.rows {
        let source = mesh.rows - 1 - row;
        for col in 0..mesh.cols {
            let p = mesh.points[source * mesh.cols + col];
            out.points.push((p.0, height - p.1));
        }
    }
    out
}

/// Transpose a mesh and swap x/y: the `warpRotate == Vrtc` construction.
fn swap_axes(mesh: &WarpMesh) -> WarpMesh {
    let mut out = WarpMesh {
        rows: mesh.cols,
        cols: mesh.rows,
        points: vec![(0.0, 0.0); mesh.points.len()],
    };
    for row in 0..mesh.rows {
        for col in 0..mesh.cols {
            let from = row * mesh.cols + col;
            let to = col * mesh.rows + row;
            let p = mesh.points[from];
            out.points[to] = (p.1, p.0);
        }
    }
    out
}

/// The pure-horizontal construction. `bend` is clamped to `[-100, 100]`; `width`
/// and `height` are the warp box the mesh spans.
fn horizontal_style_mesh(style: WarpStyle, value: f64, width: f64, height: f64) -> WarpMesh {
    let bend = value.clamp(-100.0, 100.0);
    let theta = bend.abs() * PI / 200.0;
    let displacement = 2.0 * height * bend / 100.0;
    const ZERO_BEND: f64 = 1e-9;

    match style {
        WarpStyle::ArcLower | WarpStyle::ArcUpper => {
            if bend.abs() < ZERO_BEND {
                return identity_mesh(4, 2, width as i32, height as i32);
            }
            // One edge stays put; the other arcs away for positive bend and into
            // the box for negative.
            let mut mesh = WarpMesh::new(4, 2);
            push_identity_row(&mut mesh, width, 0.0);
            push_arc_row(&mut mesh, 0.0, width, height, theta, bend < 0.0);
            if style == WarpStyle::ArcUpper {
                mirror_vertically(&mesh, height)
            } else {
                mesh
            }
        }
        WarpStyle::Fish => {
            // Flag's S-curve on the top edge, inverted on the bottom edge.
            let mut mesh = WarpMesh::new(4, 2);
            push_identity_row(&mut mesh, width, 0.0);
            push_identity_row(&mut mesh, width, height);
            mesh.points[1].1 -= displacement;
            mesh.points[2].1 += displacement;
            mesh.points[5].1 += displacement;
            mesh.points[6].1 -= displacement;
            mesh
        }
        WarpStyle::Fisheye => {
            // 4x4; the four interior points move toward their nearest corner by
            // `bend/50` (at +50 they sit exactly on the corners).
            let mut mesh = identity_mesh(4, 4, width as i32, height as i32);
            let t = bend / 50.0;
            for row in 1..=2 {
                for col in 1..=2 {
                    let index = row * 4 + col;
                    let corner_x = if col == 1 { 0.0 } else { width };
                    let corner_y = if row == 1 { 0.0 } else { height };
                    mesh.points[index].0 += t * (corner_x - mesh.points[index].0);
                    mesh.points[index].1 += t * (corner_y - mesh.points[index].1);
                }
            }
            mesh
        }
        WarpStyle::Inflate | WarpStyle::Squeeze => {
            // Quadratic 3x3 patch: corners and centre pinned, edge midpoints slide
            // by a quarter of the axis extent per 50% bend. Inflate pushes all
            // four outward; squeeze pinches the sides inward.
            let mut mesh = identity_mesh(3, 3, width as i32, height as i32);
            let dx = width * bend / 200.0;
            let dy = height * bend / 200.0;
            mesh.points[1].1 -= dy;
            mesh.points[7].1 += dy;
            if style == WarpStyle::Inflate {
                mesh.points[3].0 -= dx;
                mesh.points[5].0 += dx;
            } else {
                mesh.points[3].0 += dx;
                mesh.points[5].0 -= dx;
            }
            mesh
        }
        WarpStyle::Twist => {
            // 4x4 tangential circulation of the interior ring, direction by bend
            // sign. Orientation-invariant in Photoshop.
            let mut mesh = identity_mesh(4, 4, width as i32, height as i32);
            let move_x = width * bend.abs() / 100.0;
            let move_y = height * bend.abs() / 100.0;
            if bend > 0.0 {
                mesh.points[5].0 += move_x;
                mesh.points[6].1 += move_y;
                mesh.points[10].0 -= move_x;
                mesh.points[9].1 -= move_y;
            } else if bend < 0.0 {
                mesh.points[5].1 += move_y;
                mesh.points[6].0 -= move_x;
                mesh.points[10].1 -= move_y;
                mesh.points[9].0 += move_x;
            }
            mesh
        }
        WarpStyle::ShellLower | WarpStyle::ShellUpper => {
            if bend.abs() < ZERO_BEND {
                return identity_mesh(4, 4, width as i32, height as i32);
            }
            // Two identity rows; the 2h/3 row's endpoints rotate by theta about
            // the anchor corners; the shell edge fans out or arcs into the box.
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let mut mesh = WarpMesh::new(4, 4);
            push_identity_row(&mut mesh, width, 0.0);
            push_identity_row(&mut mesh, width, height / 3.0);
            if bend > 0.0 {
                let radius = 2.0 * height / 3.0;
                mesh.points.push((-radius * sin_theta, radius * cos_theta));
                mesh.points.push((width / 3.0, 2.0 * height / 3.0));
                mesh.points.push((2.0 * width / 3.0, 2.0 * height / 3.0));
                mesh.points
                    .push((width + radius * sin_theta, radius * cos_theta));
                push_arc_row(
                    &mut mesh,
                    -height * sin_theta,
                    width + height * sin_theta,
                    height * cos_theta,
                    theta,
                    false,
                );
            } else {
                let radius = height / 3.0;
                mesh.points
                    .push((-radius * sin_theta, height - radius * cos_theta));
                mesh.points.push((width / 3.0, 2.0 * height / 3.0));
                mesh.points.push((2.0 * width / 3.0, 2.0 * height / 3.0));
                mesh.points
                    .push((width + radius * sin_theta, height - radius * cos_theta));
                push_arc_row(&mut mesh, 0.0, width, height, theta, true);
            }
            if style == WarpStyle::ShellUpper {
                mirror_vertically(&mesh, height)
            } else {
                mesh
            }
        }
        WarpStyle::Arc => {
            if bend.abs() < ZERO_BEND {
                return identity_mesh(4, 2, width as i32, height as i32);
            }
            // Top corners swing outward about the bottom corners (radius =
            // height); both edges arc with the same central angle.
            let sin_theta = theta.sin();
            let mut mesh = WarpMesh::new(4, 2);
            push_arc_row(
                &mut mesh,
                -height * sin_theta,
                width + height * sin_theta,
                height * (1.0 - theta.cos()),
                theta,
                true,
            );
            push_arc_row(&mut mesh, 0.0, width, height, theta, true);
            if bend < 0.0 {
                mirror_vertically(&mesh, height)
            } else {
                mesh
            }
        }
        WarpStyle::Arch | WarpStyle::Bulge => {
            if bend.abs() < ZERO_BEND {
                return identity_mesh(4, 2, width as i32, height as i32);
            }
            // Arch: both edges bow the same way. Bulge: the bottom edge bows the
            // opposite way, inflating the band.
            let top_up = bend > 0.0;
            let mut mesh = WarpMesh::new(4, 2);
            push_arc_row(&mut mesh, 0.0, width, 0.0, theta, top_up);
            push_arc_row(
                &mut mesh,
                0.0,
                width,
                height,
                theta,
                if style == WarpStyle::Bulge {
                    !top_up
                } else {
                    top_up
                },
            );
            mesh
        }
        WarpStyle::Flag => {
            let mut mesh = WarpMesh::new(4, 2);
            push_identity_row(&mut mesh, width, 0.0);
            push_identity_row(&mut mesh, width, height);
            mesh.points[1].1 -= displacement;
            mesh.points[2].1 += displacement;
            mesh.points[5].1 -= displacement;
            mesh.points[6].1 += displacement;
            mesh
        }
        WarpStyle::Wave => {
            let mut mesh = WarpMesh::new(4, 3);
            push_identity_row(&mut mesh, width, 0.0);
            push_identity_row(&mut mesh, width, height / 2.0);
            push_identity_row(&mut mesh, width, height);
            // Edges stay pinned; only the middle row waves (opposite to flag).
            mesh.points[5].1 += displacement;
            mesh.points[6].1 -= displacement;
            mesh
        }
        _ => {
            // Rise: rigid column ramp on the left half of both rows.
            let mut mesh = WarpMesh::new(4, 2);
            push_identity_row(&mut mesh, width, 0.0);
            push_identity_row(&mut mesh, width, height);
            mesh.points[0].1 += displacement;
            mesh.points[1].1 += displacement;
            mesh.points[4].1 += displacement;
            mesh.points[5].1 += displacement;
            mesh
        }
    }
}

/// The control net for a named preset at `bend` percent.
///
/// `rotate_vertical` selects Photoshop's `Vrtc` construction; `Twist` ignores it
/// (Photoshop bakes identical meshes for both orientations). Returns `None` for
/// [`WarpStyle::None`]/[`WarpStyle::Custom`], a non-finite `bend`, or a
/// non-positive size.
pub fn style_mesh(
    style: WarpStyle,
    bend: f64,
    rotate_vertical: bool,
    w: i32,
    h: i32,
) -> Option<WarpMesh> {
    if !style.is_preset() || !bend.is_finite() || w <= 0 || h <= 0 {
        return None;
    }
    let (wf, hf) = (w as f64, h as f64);
    if !rotate_vertical || style == WarpStyle::Twist {
        Some(horizontal_style_mesh(style, bend, wf, hf))
    } else {
        Some(swap_axes(&horizontal_style_mesh(style, bend, hf, wf)))
    }
}
