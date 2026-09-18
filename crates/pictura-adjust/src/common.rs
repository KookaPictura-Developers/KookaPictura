use pictura_core::PixelBuffer;

use crate::types::AdjustError;

const COLOR_CHANNELS: usize = 3;
/// Rec.601 luma weights; the specs leave the exact set undocumented.
const LUMA: [f64; 3] = [0.299, 0.587, 0.114];

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub(crate) fn validate(buf: &PixelBuffer) -> Result<usize, AdjustError> {
    if buf.channels != 3 && buf.channels != 4 {
        return Err(AdjustError::Unsupported(format!(
            "channel count {} is not supported (expected 3 or 4)",
            buf.channels
        )));
    }
    let n = buf.pixel_count();
    if n == 0 {
        return Err(AdjustError::InvalidParams("empty buffer".into()));
    }
    if buf.data.len() != n * buf.channels as usize {
        return Err(AdjustError::InvalidParams(
            "buffer length does not match width*height*channels".into(),
        ));
    }
    Ok(n)
}

/// Split the planar color channels of a validated buffer. The returned slices
/// are `R`, `G`, `B`; alpha (when present) is left untouched.
pub(crate) fn planes_mut(buf: &mut PixelBuffer, n: usize) -> (&mut [u8], &mut [u8], &mut [u8]) {
    let (r, rest) = buf.data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _a) = rest.split_at_mut(n);
    (r, g, b)
}

pub(crate) fn map_lut(buf: &mut PixelBuffer, n: usize, lut: &[u8; 256]) {
    for c in 0..COLOR_CHANNELS {
        let s = c * n;
        for v in &mut buf.data[s..s + n] {
            *v = lut[*v as usize];
        }
    }
}

pub(crate) fn map_float(buf: &mut PixelBuffer, n: usize, f: impl Fn(u8) -> u8) {
    for c in 0..COLOR_CHANNELS {
        let s = c * n;
        for v in &mut buf.data[s..s + n] {
            *v = f(*v);
        }
    }
}

pub(crate) fn luma(r: f64, g: f64, b: f64) -> f64 {
    LUMA[0] * r + LUMA[1] * g + LUMA[2] * b
}

pub(crate) fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-12 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } * 60.0;
    (h.rem_euclid(360.0), s, l)
}

pub(crate) fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s <= 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    (
        hue2rgb(p, q, hk + 1.0 / 3.0),
        hue2rgb(p, q, hk),
        hue2rgb(p, q, hk - 1.0 / 3.0),
    )
}

pub(crate) fn hue2rgb(p: f64, q: f64, t: f64) -> f64 {
    let t = t.rem_euclid(1.0);
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 0.5 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

/// Monotone cubic Hermite tangents (Fritsch–Carlson limiter).
pub(crate) fn monotone_tangents(xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len();
    let mut d = vec![0.0; n - 1];
    for (i, di) in d.iter_mut().enumerate() {
        *di = (ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]);
    }
    let mut m = vec![0.0; n];
    if let Some(first) = m.first_mut() {
        *first = d[0];
    }
    if let Some(last) = m.last_mut() {
        *last = d[n - 2];
    }
    for i in 1..n - 1 {
        m[i] = (d[i - 1] + d[i]) / 2.0;
    }
    for i in 0..n - 1 {
        if d[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let a = m[i] / d[i];
            let b = m[i + 1] / d[i];
            let s = a * a + b * b;
            if s > 9.0 {
                let tau = 3.0 / s.sqrt();
                m[i] = tau * a * d[i];
                m[i + 1] = tau * b * d[i];
            }
        }
    }
    m
}

pub(crate) fn hermite_eval(xs: &[f64], ys: &[f64], ms: &[f64], x: f64) -> f64 {
    let n = xs.len();
    if x <= xs[0] {
        return ys[0];
    }
    if x >= xs[n - 1] {
        return ys[n - 1];
    }
    let mut i = 0;
    while x > xs[i + 1] {
        i += 1;
    }
    let h = xs[i + 1] - xs[i];
    let t = (x - xs[i]) / h;
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * ys[i]
        + (t3 - 2.0 * t2 + t) * h * ms[i]
        + (-2.0 * t3 + 3.0 * t2) * ys[i + 1]
        + (t3 - t2) * h * ms[i + 1]
}

pub(crate) fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub(crate) fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
