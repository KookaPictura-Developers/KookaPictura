//! Undulating warps (`FILT-040`): Ripple, Wave.
//!
//! Inverse-mapping warps with bilinear color sampling; alpha is never touched.
//! The exact generator models are closed (`docs/dev/m9-distort.md`).

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::kernel::{bilinear, invalid, to_u8, unit_f64, Edge};
use crate::{validate, FilterError, RippleSize, WaveType};

/// Periodic sinusoidal displacement; `size` sets the spatial frequency.
/// `amount` is in `-999..=999`, `0` is a bit-exact no-op. Clamp-to-edge.
pub fn ripple(buf: &mut PixelBuffer, amount: f64, size: RippleSize) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !amount.is_finite() || !(-999.0..=999.0).contains(&amount) {
        return Err(invalid(format!(
            "ripple amount {amount} must be finite and within -999..=999"
        )));
    }
    if amount == 0.0 {
        return Ok(());
    }
    // ponytail: the reference's ripple generator is closed; this is a fixed
    // axis-aligned sinusoid. Ceiling: no CS6 pixel parity. Upgrade by fitting
    // reference renders (oracle hook M9-B).
    let period = match size {
        RippleSize::Small => 8.0,
        RippleSize::Medium => 16.0,
        RippleSize::Large => 32.0,
    };
    let amp = amount * 0.1;
    let phase = std::f64::consts::TAU / period;
    let (w, h) = (buf.width as usize, buf.height as usize);
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    for y in 0..h {
        for x in 0..w {
            let dx = amp * (phase * y as f64).sin();
            let dy = amp * (phase * x as f64).sin();
            for c in 0..planes {
                let plane = &src[c * n..c * n + n];
                let v = bilinear(plane, w, h, x as f64 + dx, y as f64 + dy, Edge::Clamp);
                buf.data[c * n + y * w + x] = to_u8(v);
            }
        }
    }
    Ok(())
}

/// Sum of `generators` sinusoidal generators along the x axis.
///
/// Each draws a period from `wavelength`, amplitude from `amplitude`, and a
/// phase from `[0, TAU)` via `ChaCha8Rng::seed_from_u64`. `scale` is the
/// axis-wise displacement percent; `repeat_edge` picks clamp (`true`) or wrap
/// (`false`). Same seed and parameters are bit-identical.
#[allow(clippy::too_many_arguments)]
pub fn wave(
    buf: &mut PixelBuffer,
    generators: u32,
    wavelength: (f64, f64),
    amplitude: (f64, f64),
    kind: WaveType,
    scale: (f64, f64),
    seed: u64,
    repeat_edge: bool,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    validate_wave(generators, wavelength, amplitude, scale)?;
    // ponytail: the reference's multi-generator model is closed; this sums axis-aligned
    // sinusoids with uniform draws. Ceiling: no CS6 pixel parity. Upgrade by
    // fitting reference renders (oracle hook M9-B).
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut gens = Vec::with_capacity(generators as usize);
    for _ in 0..generators {
        let period = wavelength.0 + (wavelength.1 - wavelength.0) * unit_f64(&mut rng);
        let phase = std::f64::consts::TAU * unit_f64(&mut rng);
        let amp = amplitude.0 + (amplitude.1 - amplitude.0) * unit_f64(&mut rng);
        gens.push((amp, period, phase));
    }
    let (w, h) = (buf.width as usize, buf.height as usize);
    let planes = (buf.channels as usize).min(3);
    let edge = if repeat_edge { Edge::Clamp } else { Edge::Wrap };
    // The field depends only on x, so build the column displacement once.
    let mut offs = vec![(0.0f64, 0.0f64); w];
    for (x, off) in offs.iter_mut().enumerate() {
        let mut s = 0.0;
        for &(amp, period, phase) in &gens {
            s += amp * shape(kind, std::f64::consts::TAU * x as f64 / period + phase);
        }
        *off = (s * scale.0 / 100.0, s * scale.1 / 100.0);
    }
    let src = buf.data.clone();
    for y in 0..h {
        for (x, &(dx, dy)) in offs.iter().enumerate() {
            for c in 0..planes {
                let plane = &src[c * n..c * n + n];
                let v = bilinear(plane, w, h, x as f64 + dx, y as f64 + dy, edge);
                buf.data[c * n + y * w + x] = to_u8(v);
            }
        }
    }
    Ok(())
}

fn validate_wave(
    generators: u32,
    wavelength: (f64, f64),
    amplitude: (f64, f64),
    scale: (f64, f64),
) -> Result<(), FilterError> {
    if !(1..=999).contains(&generators) {
        return Err(invalid(format!(
            "wave generators {generators} outside 1..=999"
        )));
    }
    for (name, r) in [("wavelength", wavelength), ("amplitude", amplitude)] {
        if !r.0.is_finite() || !r.1.is_finite() {
            return Err(invalid(format!("wave {name} must be finite")));
        }
        if !(1.0..=998.0).contains(&r.0) {
            return Err(invalid(format!("wave {name} min {} outside 1..=998", r.0)));
        }
        if r.1 < r.0 + 1.0 {
            return Err(invalid(format!("wave {name} max must be >= min + 1")));
        }
    }
    for s in [scale.0, scale.1] {
        if !s.is_finite() || !(1.0..=100.0).contains(&s) {
            return Err(invalid(format!("wave scale {s} outside 1..=100")));
        }
    }
    Ok(())
}

fn shape(kind: WaveType, theta: f64) -> f64 {
    let u = theta.rem_euclid(std::f64::consts::TAU);
    match kind {
        WaveType::Sine => theta.sin(),
        WaveType::Triangle => 1.0 - 4.0 * (u / std::f64::consts::TAU - 0.5).abs(),
        WaveType::Square => {
            if u < std::f64::consts::PI {
                1.0
            } else {
                -1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WL: (f64, f64) = (10.0, 40.0);
    const AMP: (f64, f64) = (5.0, 15.0);
    const SC: (f64, f64) = (100.0, 50.0);
    const NWL: (f64, f64) = (10.0, 11.0);
    const NAMP: (f64, f64) = (20.0, 21.0);
    const NSC: (f64, f64) = (100.0, 1.0);

    fn buf3(w: u32, h: u32, px: &[[u8; 3]]) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 3);
        let n = px.len();
        for (i, p) in px.iter().enumerate() {
            for (c, &v) in p.iter().enumerate() {
                b.data[c * n + i] = v;
            }
        }
        b
    }

    fn buf4(w: u32, h: u32, px: &[[u8; 4]]) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 4);
        let n = px.len();
        for (i, p) in px.iter().enumerate() {
            for (c, &v) in p.iter().enumerate() {
                b.data[c * n + i] = v;
            }
        }
        b
    }

    /// Deterministic high-frequency pattern so a warp cannot be invariant.
    fn ramp(w: u32, h: u32) -> PixelBuffer {
        let px: Vec<[u8; 3]> = (0..(w * h) as usize)
            .map(|i| [i as u8, (i * 5) as u8, (i * 11) as u8])
            .collect();
        buf3(w, h, &px)
    }

    fn wv(base: &PixelBuffer, g: u32, kind: WaveType, seed: u64, repeat: bool) -> PixelBuffer {
        let mut out = base.clone();
        wave(&mut out, g, WL, AMP, kind, SC, seed, repeat).unwrap();
        out
    }

    fn narrow(base: &PixelBuffer, g: u32, seed: u64) -> PixelBuffer {
        let mut out = base.clone();
        wave(&mut out, g, NWL, NAMP, WaveType::Sine, NSC, seed, true).unwrap();
        out
    }

    #[test]
    fn ripple_noop_displaces_and_sizes_differ() {
        let base = ramp(24, 24);
        let mut noop = base.clone();
        ripple(&mut noop, 0.0, RippleSize::Medium).unwrap();
        assert_eq!(noop.data, base.data);
        let (mut small, mut large) = (base.clone(), base.clone());
        ripple(&mut small, 100.0, RippleSize::Small).unwrap();
        ripple(&mut large, 100.0, RippleSize::Large).unwrap();
        assert_ne!(small.data, base.data);
        assert_ne!(small.data, large.data);
    }

    #[test]
    fn wave_is_seed_deterministic_and_moves_pixels() {
        let base = ramp(24, 24);
        let a = wv(&base, 5, WaveType::Sine, 7, true);
        assert_eq!(a.data, wv(&base, 5, WaveType::Sine, 7, true).data);
        assert_ne!(a.data, wv(&base, 5, WaveType::Sine, 8, true).data);
        assert_eq!(narrow(&base, 1, 42).data, narrow(&base, 1, 42).data);
        assert_ne!(narrow(&base, 1, 42).data, base.data);
    }

    #[test]
    fn wave_edge_modes_types_and_validation() {
        let base = ramp(32, 8);
        let a = wv(&base, 3, WaveType::Sine, 1, true);
        assert_ne!(a.data, wv(&base, 3, WaveType::Sine, 1, false).data);
        let sine = wv(&base, 2, WaveType::Sine, 3, true);
        assert_ne!(sine.data, wv(&base, 2, WaveType::Triangle, 3, true).data);
        assert_ne!(sine.data, wv(&base, 2, WaveType::Square, 3, true).data);

        let reject = |g: u32, wl: (f64, f64), amp: (f64, f64), sc: (f64, f64)| {
            let mut b = base.clone();
            assert!(wave(&mut b, g, wl, amp, WaveType::Sine, sc, 1, true).is_err());
            assert_eq!(b.data, base.data);
        };
        let (wl, amp, ok) = ((10.0, 20.0), (5.0, 15.0), (100.0, 100.0));
        reject(0, wl, amp, ok);
        reject(1000, wl, amp, ok);
        reject(5, (20.0, 20.5), amp, ok);
        reject(5, wl, (20.0, 20.5), ok);
        reject(5, wl, amp, (0.0, 100.0));
        reject(5, wl, amp, (100.0, 101.0));
    }

    #[test]
    fn alpha_preserved_and_tiny_images_do_not_panic() {
        let px: Vec<[u8; 4]> = (0..24 * 24)
            .map(|i| [i as u8, (i * 3) as u8, (i * 7) as u8, (i * 11) as u8])
            .collect();
        let base = buf4(24, 24, &px);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let mut r = base.clone();
        ripple(&mut r, 400.0, RippleSize::Small).unwrap();
        assert_eq!(&r.data[3 * n..], &alpha[..]);
        let w = wv(&base, 4, WaveType::Square, 9, false);
        assert_eq!(&w.data[3 * n..], &alpha[..]);

        let one = buf4(1, 1, &[[100, 150, 200, 255]]);
        let mut r1 = one.clone();
        ripple(&mut r1, 500.0, RippleSize::Large).unwrap();
        assert_eq!(r1.data, one.data);
        assert_eq!(wv(&one, 999, WaveType::Triangle, 1, false).data, one.data);

        for &(w, h) in &[(1u32, 8u32), (8, 1)] {
            let px: Vec<[u8; 3]> = (0..(w * h) as usize)
                .map(|i| [i as u8, 0, 255 - i as u8])
                .collect();
            let base = buf3(w, h, &px);
            let mut a = base.clone();
            assert!(ripple(&mut a, 250.0, RippleSize::Medium).is_ok());
            wv(&base, 2, WaveType::Sine, 3, true);
        }
    }
}
