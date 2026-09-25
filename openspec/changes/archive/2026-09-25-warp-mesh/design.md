# Design: warp-mesh

## Context

The similarity op (`transform.rs`) already has the refusal contract, the
channel-less smart-object materialization, the integer-bounding-box rule, the
`MAX_RESULT_PIXELS` guard, and a bilinear `resample_plane` driven by a
`PlaneMap` (a `forward`/`inverse` pair). The warp op reuses that skeleton; only
the map differs.

## Goals / Non-Goals

**Goals**

- A deterministic, testable custom mesh warp with an exact identity.

**Non-Goals**

- Named presets, `Bend`/`X`/`Y` preset geometry, the interactive overlay, and the
  menu command.

## Decisions

**Mesh.** `WarpMesh { rows: usize, cols: usize, points: Vec<(f64, f64)> }` in the
source rect's local space, row-major; the shipped default is 4×4 (16 points).
`identity_mesh(cols, rows, w, h)` is the uniform net `((i/(cols-1))·w,
(j/(rows-1))·h)`.

**Surface.** Degree-3 tensor-product Bézier: `B(u,v) = ΣᵢΣⱼ Pᵢⱼ·Bᵢ³(u)·Bⱼ³(v)`
with `Bᵢ³` the Bernstein basis, `u,v ∈ [0,1]`. A uniform net is the identity
because the Bézier of collinear evenly spaced controls is linear.

**Distortion.** `distort_h`/`distort_v` (percent) scale each mesh row about its
two edge points' midpoint by `1 + (2v−1)·distort_v/100` and each column likewise
by `distort_h` — the options-bar "Distortion" behaviour documented by Patchy.

**Rasterization.** Subdivide `[0,1]²` into `N×N` cells (`N = 16`); for each cell,
split its four forward-mapped corners into two triangles and inverse-map each
destination pixel barycentrically to `(u,v)`; sample the source bilinearly with
the existing edge-clamp/out-of-source→0 rule. Cells are painted in order (a
self-intersecting mesh lets later cells overwrite, marked `ponytail:`). The
destination bounding box is the integer bbox of the forward lattice, guarded by
`MAX_RESULT_PIXELS`.

**Refusal.** Identical to `transform_layer`, plus a non-finite mesh point, a
non-positive `rows`/`cols` below 2, or a point count not equal to `rows·cols`.

## Risks / Trade-offs

- [No Photoshop oracle] → parity is behavioral; tests pin identity, corner
  fixing, determinism, and refusals. Preset exactness is explicitly deferred.
- [Forward scatter + painter's order] → deterministic; overlaps only occur for a
  self-intersecting mesh, which the UI normally prevents.
- [Cell approximation of the exact surface] → `N=16` bounds the error; raise it if
  a golden comparison ever matters.

## Migration Plan

Additive; revert is a revert.
