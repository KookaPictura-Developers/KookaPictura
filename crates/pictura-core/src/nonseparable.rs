//! The W3C Compositing and Blending Level 1 non-separable helpers (PDF/CSS
//! triplet math) behind the Hue, Saturation, Color, and Luminosity modes,
//! shared by the compositor and the paint tools that blend by those modes.

#[inline]
pub fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

#[inline]
pub fn sat(c: [f32; 3]) -> f32 {
    let max = c.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let min = c.iter().copied().fold(f32::INFINITY, f32::min);
    max - min
}

#[inline]
pub fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

#[inline]
fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c.iter().copied().fold(f32::INFINITY, f32::min);
    let x = c.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut out = c;
    if n < 0.0 {
        let d = l - n;
        if d != 0.0 {
            out = [
                l + (out[0] - l) * l / d,
                l + (out[1] - l) * l / d,
                l + (out[2] - l) * l / d,
            ];
        }
    }
    if x > 1.0 {
        let d = x - l;
        if d != 0.0 {
            out = [
                l + (out[0] - l) * (1.0 - l) / d,
                l + (out[1] - l) * (1.0 - l) / d,
                l + (out[2] - l) * (1.0 - l) / d,
            ];
        }
    }
    out
}

#[inline]
pub fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let (mut imin, mut imax) = (0usize, 0usize);
    for i in 1..3 {
        if c[i] < c[imin] {
            imin = i;
        }
        if c[i] > c[imax] {
            imax = i;
        }
    }
    let cmin = c[imin];
    let cmax = c[imax];
    if cmax > cmin {
        let imid = 3 - imin - imax;
        let mut out = c;
        out[imid] = (c[imid] - cmin) * s / (cmax - cmin);
        out[imax] = s;
        out[imin] = 0.0;
        out
    } else {
        [0.0, 0.0, 0.0]
    }
}
