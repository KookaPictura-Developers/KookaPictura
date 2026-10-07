//! The photorust filter engine (#183): the Artistic, Brush Strokes, Sketch,
//! Texture, Pixelate, Distort, Stylize, and Noise filters that replace
//! Kooka's own, run on an interleaved [`Pixmap`] that `apply` converts to and
//! from the planar buffer.
//!
//! Ported from perfecto25/photorust (`core/src/filters/`), GPL-3.0.

// photorust's idioms stay as written so the later ports (#222-#227) diff
// cleanly against upstream; these two lints flag style only.
#![allow(clippy::chunks_exact_to_as_chunks, clippy::needless_range_loop)]

pub(crate) mod artistic;
pub(crate) mod brush_strokes;
pub(crate) mod convolve;
pub(crate) mod dispatch;
pub(crate) mod distort;
pub(crate) mod pixelate;
pub(crate) mod pixmap;
pub(crate) mod segment;
pub(crate) mod sketch;
pub(crate) mod stylize;
pub(crate) mod texture;

pub(crate) use pixmap::{Pixmap, Rgba8};

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

static SEED: AtomicU32 = AtomicU32::new(0);
static SEED_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` with `seed` mixed into every photorust noise primitive. photorust
/// seeds its patterns from pixel coordinates alone; Kooka's filters take a
/// user `seed` that must re-roll them, and the primitives run on rayon
/// workers that a thread-local would not reach. Seed 0 is photorust's own
/// pattern.
/// ponytail: one process-wide seed, so seeded filter runs are serialized by
/// the lock; thread a seed parameter through the primitives if concurrent
/// filter runs ever matter.
pub(crate) fn with_seed<R>(seed: u64, f: impl FnOnce() -> R) -> R {
    let _guard = SEED_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    SEED.store(fold_seed(seed), Ordering::SeqCst);
    let out = f();
    SEED.store(0, Ordering::SeqCst);
    out
}

/// The seed of the running filter (0 outside [`with_seed`]).
#[inline]
pub(crate) fn seed() -> u32 {
    SEED.load(Ordering::Relaxed)
}

/// A nonzero 32-bit mix of a user seed, so seed 0 is not photorust's default.
fn fold_seed(seed: u64) -> u32 {
    let folded = hash2(seed as u32, (seed >> 32) as u32 ^ 0x5EED);
    folded.max(1)
}

/// Deterministic noise added to every visible pixel, `amount` a percentage
/// of the tonal range (to 400), uniform or Gaussian, per channel or
/// monochromatic; the pattern follows the [`with_seed`] seed.
pub(crate) fn add_noise(pixmap: &mut Pixmap, amount: f32, monochromatic: bool, gaussian: bool) {
    let magnitude = (amount.clamp(0.0, 400.0) / 100.0) * 127.5;
    if magnitude <= 0.0 {
        return;
    }
    let salt = seed();
    let width = pixmap.width();
    for y in 0..pixmap.height() {
        for x in 0..width {
            let base = hash2(x ^ salt, y.wrapping_add(salt.rotate_left(16)));
            let jitter = |salt: u32| -> i32 {
                let h = hash2(base.wrapping_add(salt), salt);
                let u = (h % 2001) as f32 / 1000.0 - 1.0;
                if !gaussian {
                    return (u * magnitude) as i32;
                }
                // Three summed hashes make a near-enough bell.
                let v = (hash2(h, salt.wrapping_add(101)) % 2001) as f32 / 1000.0 - 1.0;
                let w = (hash2(h, salt.wrapping_add(211)) % 2001) as f32 / 1000.0 - 1.0;
                ((u + v + w) / 3.0 * 1.732 * magnitude) as i32
            };
            let px = pixmap.get(x as i32, y as i32);
            if px.a == 0 {
                continue;
            }
            let (dr, dg, db) = if monochromatic {
                let d = jitter(0);
                (d, d, d)
            } else {
                (jitter(0), jitter(1), jitter(2))
            };
            pixmap.set(
                x as i32,
                y as i32,
                Rgba8::new(
                    (px.r as i32 + dr).clamp(0, 255) as u8,
                    (px.g as i32 + dg).clamp(0, 255) as u8,
                    (px.b as i32 + db).clamp(0, 255) as u8,
                    px.a,
                ),
            );
        }
    }
}

/// Cheap integer hash for reproducible per-pixel noise.
#[inline]
fn hash2(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_F491);
    h ^= h >> 13;
    h
}
