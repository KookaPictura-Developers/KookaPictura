//! Content-Aware healing: patch synthesis. An onion peel fills the hole from
//! its boundary inward with the centre of the best-matching nearby patch, then
//! PatchMatch search-and-vote refines it globally so neighbouring pixels agree
//! instead of forming a mosaic. [`Adaptation`] sets the patch size and the
//! search reach; Medium gives a 5×5 patch searched 24 pixels out.

use super::{laplace_fill, Adaptation};

const REFINE_PASSES: usize = 8;
/// Turns a patch's mean squared difference into a voting weight.
const VOTE_FALLOFF: f32 = 250.0;
/// Coarsest candidate spacing; sparser matching breaks the fill into blocks.
const MAX_STRIDE: i32 = 4;

pub(super) fn content_aware_fill(
    rgba: &[[f32; 4]],
    hole: &[bool],
    w: usize,
    h: usize,
    adaptation: Adaptation,
) -> Vec<[f32; 4]> {
    let (patch, search) = adaptation.patch_and_search();
    let mut out = onion_peel_fill(rgba, hole, w, h, patch, search);
    patchmatch_refine(&mut out, hole, w, h, patch);
    out
}

fn onion_peel_fill(
    rgba: &[[f32; 4]],
    hole: &[bool],
    w: usize,
    h: usize,
    patch: i32,
    search: i32,
) -> Vec<[f32; 4]> {
    let mut out = rgba.to_vec();
    let mut unknown = hole.to_vec();
    let mut remaining = unknown.iter().filter(|&&u| u).count();
    // Candidates thin out as the hole grows, holding the cost roughly constant.
    let stride = (((remaining as f32) / 4_000.0).sqrt().ceil() as i32).clamp(1, MAX_STRIDE);
    let mut passes = 0;
    while remaining > 0 && passes < w.max(h) + 2 {
        passes += 1;
        // This pass's boundary layer, resolved against a snapshot so pixels
        // filled during the pass do not become sources within it.
        let layer: Vec<(i32, i32)> = (0..h as i32)
            .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = y as usize * w + x as usize;
                unknown[i] && touches_known(&unknown, w, h, x as usize, y as usize)
            })
            .collect();
        if layer.is_empty() {
            break;
        }
        let resolved: Vec<[f32; 4]> = layer
            .iter()
            .map(|&(x, y)| best_match(&out, &unknown, w, h, (x, y), stride, (patch, search)))
            .collect();
        for (&(x, y), value) in layer.iter().zip(resolved) {
            let i = y as usize * w + x as usize;
            out[i] = value;
            unknown[i] = false;
            remaining -= 1;
        }
    }
    // Anything unreachable falls back to the smooth solve.
    if remaining > 0 {
        let leftover = laplace_fill(&out, &unknown, w, h);
        for i in 0..out.len() {
            if unknown[i] {
                out[i] = leftover[i];
            }
        }
    }
    out
}

/// The centre of the nearby known patch whose known surroundings best match
/// those of hole pixel `(hx, hy)`.
fn best_match(
    out: &[[f32; 4]],
    unknown: &[bool],
    w: usize,
    h: usize,
    (hx, hy): (i32, i32),
    stride: i32,
    (patch, search): (i32, i32),
) -> [f32; 4] {
    let mut best = f32::MAX;
    let mut best_value = out[hy as usize * w + hx as usize];
    let x1 = (hx + search).min(w as i32 - 1 - patch);
    let y1 = (hy + search).min(h as i32 - 1 - patch);
    let mut sy = (hy - search).max(patch);
    while sy <= y1 {
        let mut sx = (hx - search).max(patch);
        while sx <= x1 {
            if !unknown[sy as usize * w + sx as usize] {
                let mut cost = 0.0f32;
                let mut counted = 0;
                for dy in -patch..=patch {
                    for dx in -patch..=patch {
                        let (px, py) = (hx + dx, hy + dy);
                        if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                            continue;
                        }
                        let pi = py as usize * w + px as usize;
                        if unknown[pi] {
                            continue;
                        }
                        let qi = (sy + dy) as usize * w + (sx + dx) as usize;
                        for (&a, &b) in out[pi][..3].iter().zip(&out[qi][..3]) {
                            let d = a - b;
                            cost += d * d;
                        }
                        counted += 1;
                    }
                }
                if counted > 0 && cost / (counted as f32) < best {
                    best = cost / counted as f32;
                    best_value = out[sy as usize * w + sx as usize];
                }
            }
            sx += stride;
        }
        sy += stride;
    }
    best_value
}

/// PatchMatch passes: improve each hole pixel's source patch from what its
/// neighbours chose and from random guesses at shrinking radius (fixed seed, so
/// the same stroke heals the same way), then recolour every hole pixel from all
/// the patches covering it, weighted by how well each matched.
fn patchmatch_refine(out: &mut [[f32; 4]], hole: &[bool], w: usize, h: usize, patch: i32) {
    let holes: Vec<(i32, i32)> = (0..h as i32)
        .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| hole[y as usize * w + x as usize])
        .collect();
    let usable = |x: i32, y: i32| -> bool {
        x >= patch
            && y >= patch
            && x < w as i32 - patch
            && y < h as i32 - patch
            && !hole[y as usize * w + x as usize]
    };
    let sources: Vec<(i32, i32)> = (patch..h as i32 - patch)
        .flat_map(|y| (patch..w as i32 - patch).map(move |x| (x, y)))
        .filter(|&(x, y)| usable(x, y))
        .collect();
    if holes.is_empty() || sources.is_empty() {
        return;
    }
    let mut slot = vec![usize::MAX; w * h];
    for (i, &(x, y)) in holes.iter().enumerate() {
        slot[y as usize * w + x as usize] = i;
    }
    let mut rng: u32 = 0x2545_F491;
    let mut roll = |limit: u32| -> u32 {
        rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (rng >> 8) % limit.max(1)
    };
    let mut nnf: Vec<(i32, i32)> = (0..holes.len())
        .map(|_| sources[roll(sources.len() as u32) as usize])
        .collect();
    let mut quality = vec![f32::MAX; holes.len()];
    let mut accumulator = vec![[0.0f32; 4]; w * h];
    let mut weights = vec![0.0f32; w * h];

    for pass in 0..REFINE_PASSES {
        // Alternate the scan direction so matches propagate both ways.
        let forward = pass % 2 == 0;
        let step: i32 = if forward { -1 } else { 1 };
        for k in 0..holes.len() {
            let i = if forward { k } else { holes.len() - 1 - k };
            let (hx, hy) = holes[i];
            let mut best = patch_cost(out, w, h, patch, (hx, hy), nnf[i]);
            for (nx, ny) in [(hx + step, hy), (hx, hy + step)] {
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let n = slot[ny as usize * w + nx as usize];
                if n == usize::MAX {
                    continue;
                }
                let candidate = (nnf[n].0 + (hx - nx), nnf[n].1 + (hy - ny));
                if usable(candidate.0, candidate.1) {
                    let cost = patch_cost(out, w, h, patch, (hx, hy), candidate);
                    if cost < best {
                        best = cost;
                        nnf[i] = candidate;
                    }
                }
            }
            let mut radius = (w.max(h) as i32 / 2).max(1);
            while radius >= 1 {
                let span = (radius * 2 + 1) as u32;
                let candidate = (
                    nnf[i].0 + roll(span) as i32 - radius,
                    nnf[i].1 + roll(span) as i32 - radius,
                );
                if usable(candidate.0, candidate.1) {
                    let cost = patch_cost(out, w, h, patch, (hx, hy), candidate);
                    if cost < best {
                        best = cost;
                        nnf[i] = candidate;
                    }
                }
                radius /= 2;
            }
            quality[i] = best;
        }

        accumulator.fill([0.0; 4]);
        weights.fill(0.0);
        for (i, &(hx, hy)) in holes.iter().enumerate() {
            let (sx, sy) = nnf[i];
            let weight = 1.0 / (1.0 + quality[i] / VOTE_FALLOFF);
            for dy in -patch..=patch {
                for dx in -patch..=patch {
                    let (tx, ty) = (hx + dx, hy + dy);
                    if tx < 0 || ty < 0 || tx >= w as i32 || ty >= h as i32 {
                        continue;
                    }
                    let target = ty as usize * w + tx as usize;
                    if !hole[target] {
                        continue;
                    }
                    let sample = (sy + dy) as usize * w + (sx + dx) as usize;
                    for c in 0..4 {
                        accumulator[target][c] += out[sample][c] * weight;
                    }
                    weights[target] += weight;
                }
            }
        }
        for i in 0..w * h {
            if hole[i] && weights[i] > 0.0 {
                out[i] = accumulator[i].map(|v| v / weights[i]);
            }
        }
    }
}

/// Mean squared RGB difference between the patches around `a` and `b`.
fn patch_cost(
    out: &[[f32; 4]],
    w: usize,
    h: usize,
    patch: i32,
    a: (i32, i32),
    b: (i32, i32),
) -> f32 {
    let mut cost = 0.0f32;
    let mut counted = 0;
    for dy in -patch..=patch {
        for dx in -patch..=patch {
            let (px, py) = (a.0 + dx, a.1 + dy);
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                continue;
            }
            let p = py as usize * w + px as usize;
            let q = (b.1 + dy) as usize * w + (b.0 + dx) as usize;
            for (&x, &y) in out[p][..3].iter().zip(&out[q][..3]) {
                let d = x - y;
                cost += d * d;
            }
            counted += 1;
        }
    }
    if counted == 0 {
        f32::MAX
    } else {
        cost / counted as f32
    }
}

fn touches_known(unknown: &[bool], w: usize, h: usize, x: usize, y: usize) -> bool {
    (x > 0 && !unknown[y * w + x - 1])
        || (x + 1 < w && !unknown[y * w + x + 1])
        || (y > 0 && !unknown[(y - 1) * w + x])
        || (y + 1 < h && !unknown[(y + 1) * w + x])
}
