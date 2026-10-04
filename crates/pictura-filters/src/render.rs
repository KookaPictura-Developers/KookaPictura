//! Render family: Clouds, Difference Clouds, Fibers, Lens Flare, Lighting
//! Effects (`FILT-060`).
//!
//! All are classified **no-equivalent**: the reference's noise and flare models
//! are closed, so these are deterministic approximations verified by property
//! tests (range, determinism, monotonicity), not delta fitting against
//! reference output. Clouds/Fibers replace the color planes from a seeded
//! value-noise field; Lens Flare adds on top of the existing pixels. Alpha is
//! never modified.

use pictura_core::PixelBuffer;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

use crate::{kernel::unit_f64, validate, FilterError};

/// Clouds: base lattice cells across the image.
const CLOUD_CELLS: f64 = 8.0;
/// fBm octaves, lacunarity and gain shared by the noise-driven filters.
const OCTAVES: usize = 5;
const LACUNARITY: f64 = 2.0;
const GAIN: f64 = 0.5;
/// Fibers: horizontal cell density as a fraction of `strength` (fibers run
/// along x), roughened further by Variance.
const FIBER_X_FREQ: f64 = 0.05;

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Seeded lattice value noise: a pseudo-random value per integer node from a
/// ChaCha-seeded table, bilinear interpolation over smoothstep-eased
/// fractional parts, and 5-octave fBm (lacunarity 2, gain 0.5) normalized to
/// [0, 1]. Deterministic for a given seed and independent of buffer size.
///
/// ponytail: the reference's noise is closed and lattice value noise shows grid
/// artifacts; switch to gradient/Perlin noise if the look ever matters.
struct ValueNoise {
    perm: [u8; 512],
    vals: [f64; 256],
}

impl ValueNoise {
    fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut half = [0u8; 256];
        rng.fill_bytes(&mut half);
        let mut noise = ValueNoise {
            perm: [0u8; 512],
            vals: [0.0; 256],
        };
        noise.perm[..256].copy_from_slice(&half);
        noise.perm[256..].copy_from_slice(&half);
        for v in &mut noise.vals {
            *v = unit_f64(&mut rng);
        }
        noise
    }

    fn node(&self, ix: i64, iy: i64, octave: usize) -> f64 {
        let x = (ix as usize) & 255;
        let y = ((iy as usize).wrapping_add(octave * 97)) & 255;
        let h = self.perm[self.perm[x] as usize + self.perm[y] as usize];
        self.vals[h as usize]
    }

    fn octave(&self, x: f64, y: f64, octave: usize) -> f64 {
        let ix = x.floor() as i64;
        let iy = y.floor() as i64;
        let tx = smoothstep(x - ix as f64);
        let ty = smoothstep(y - iy as f64);
        let v00 = self.node(ix, iy, octave);
        let v10 = self.node(ix + 1, iy, octave);
        let v01 = self.node(ix, iy + 1, octave);
        let v11 = self.node(ix + 1, iy + 1, octave);
        v00 + (v10 - v00) * tx + (v01 - v00) * ty + (v00 - v10 - v01 + v11) * tx * ty
    }

    fn fbm(&self, x: f64, y: f64) -> f64 {
        let mut sum = 0.0;
        let mut amp = 1.0;
        let mut norm = 0.0;
        let mut fx = x;
        let mut fy = y;
        for o in 0..OCTAVES {
            sum += amp * self.octave(fx, fy, o);
            norm += amp;
            amp *= GAIN;
            fx *= LACUNARITY;
            fy *= LACUNARITY;
        }
        sum / norm
    }
}

/// The normalized Clouds field (starker contrast already applied), row-major.
fn cloud_field(w: usize, h: usize, starker: bool, seed: u64) -> Vec<f64> {
    let noise = ValueNoise::new(seed);
    let mut field = Vec::with_capacity(w * h);
    for y in 0..h {
        let v = (y as f64 + 0.5) / h as f64;
        for x in 0..w {
            let u = (x as f64 + 0.5) / w as f64;
            let mut f = noise.fbm(CLOUD_CELLS * u, CLOUD_CELLS * v);
            if starker {
                f = smoothstep(f);
            }
            field.push(f);
        }
    }
    field
}

fn lerp_channel(a: u8, b: u8, f: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * f).round() as u8
}

/// Clouds: replace the color planes with the color ramp mapped through the
/// seeded fBm field (`starker` applies a smoothstep contrast curve first).
pub fn clouds(
    buf: &mut PixelBuffer,
    color_a: [u8; 3],
    color_b: [u8; 3],
    starker: bool,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = cloud_field(w, h, starker, seed);
    for (p, &f) in field.iter().enumerate() {
        for c in 0..planes {
            buf.data[c * n + p] = lerp_channel(color_a[c], color_b[c], f);
        }
    }
    Ok(())
}

/// Difference Clouds: same field as [`clouds`], then set each color sample to
/// the absolute difference between the existing pixel and the mapped cloud.
pub fn difference_clouds(
    buf: &mut PixelBuffer,
    color_a: [u8; 3],
    color_b: [u8; 3],
    starker: bool,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = cloud_field(w, h, starker, seed);
    for (p, &f) in field.iter().enumerate() {
        for c in 0..planes {
            let cloud = lerp_channel(color_a[c], color_b[c], f) as i32;
            let existing = buf.data[c * n + p] as i32;
            buf.data[c * n + p] = existing.abs_diff(cloud) as u8;
        }
    }
    Ok(())
}

/// Fibers: replace the color planes with an x-elongated noise field mapped
/// through the color ramp. `variance` (0..=100) sets per-run color variation,
/// `strength` (1..=100) the fiber definition via the vertical cell density.
pub fn fibers(
    buf: &mut PixelBuffer,
    variance: f64,
    strength: f64,
    color_a: [u8; 3],
    color_b: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(0.0..=100.0).contains(&variance) {
        return Err(FilterError::InvalidParams(format!(
            "fibers variance {variance} out of range 0..=100"
        )));
    }
    if !(1.0..=100.0).contains(&strength) {
        return Err(FilterError::InvalidParams(format!(
            "fibers strength {strength} out of range 1..=100"
        )));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let fx = (FIBER_X_FREQ * strength).max(0.25) * (1.0 + variance / 25.0);
    let fy = strength;
    let noise = ValueNoise::new(seed);
    let mut jitter_rng = ChaCha8Rng::seed_from_u64(seed ^ 0xf1be_7a1e);
    for y in 0..h {
        let v = (y as f64 + 0.5) / h as f64;
        let jitter = (unit_f64(&mut jitter_rng) - 0.5) * variance / 100.0;
        for x in 0..w {
            let u = (x as f64 + 0.5) / w as f64;
            let f = (noise.fbm(fx * u, fy * v) + jitter).clamp(0.0, 1.0);
            for c in 0..planes {
                buf.data[c * n + y * w + x] = lerp_channel(color_a[c], color_b[c], f);
            }
        }
    }
    Ok(())
}

/// Lens model for the Lens Flare ghost chain and starburst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensType {
    Zoom,
    Prime35,
    Prime105,
    MoviePrime,
}

struct Ghost {
    /// Position along the center→mirror axis (0 = center, 1 = mirrored point).
    along: f64,
    /// Radius as a fraction of the half image diagonal.
    radius: f64,
    /// Peak add relative to full scale, before the brightness factor.
    alpha: f64,
}

/// (spoke count, perpendicular ghost offset in image heights, ghosts)
fn lens_model(lens: LensType) -> (usize, f64, &'static [Ghost]) {
    match lens {
        LensType::Zoom => (
            6,
            0.0,
            &[
                Ghost {
                    along: 0.4,
                    radius: 0.10,
                    alpha: 0.22,
                },
                Ghost {
                    along: 0.8,
                    radius: 0.12,
                    alpha: 0.20,
                },
                Ghost {
                    along: 1.2,
                    radius: 0.14,
                    alpha: 0.18,
                },
                Ghost {
                    along: 1.6,
                    radius: 0.16,
                    alpha: 0.16,
                },
            ],
        ),
        LensType::Prime35 => (
            4,
            0.0,
            &[
                Ghost {
                    along: 0.5,
                    radius: 0.06,
                    alpha: 0.18,
                },
                Ghost {
                    along: 0.85,
                    radius: 0.08,
                    alpha: 0.16,
                },
                Ghost {
                    along: 1.2,
                    radius: 0.10,
                    alpha: 0.14,
                },
            ],
        ),
        LensType::Prime105 => (
            8,
            0.05,
            &[
                Ghost {
                    along: 0.7,
                    radius: 0.09,
                    alpha: 0.20,
                },
                Ghost {
                    along: 1.3,
                    radius: 0.11,
                    alpha: 0.16,
                },
            ],
        ),
        LensType::MoviePrime => (
            4,
            0.0,
            &[
                Ghost {
                    along: 0.6,
                    radius: 0.08,
                    alpha: 0.15,
                },
                Ghost {
                    along: 1.1,
                    radius: 0.10,
                    alpha: 0.13,
                },
            ],
        ),
    }
}

/// Lens Flare: additive core, ghost chain and starburst rays, clamped to
/// white. `brightness` is a percentage (10..=300); `center` is in unit
/// coordinates and is clamped, not rejected. Deterministic — no seed.
///
/// ponytail: the reference's flare model is closed; these amplitudes are an artistic
/// guess. Per-pixel exp/atan2 over every contributor — add per-contributor
/// bounding boxes if large documents get slow.
pub fn lens_flare(
    buf: &mut PixelBuffer,
    brightness: f64,
    center: (f64, f64),
    lens: LensType,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(10.0..=300.0).contains(&brightness) {
        return Err(FilterError::InvalidParams(format!(
            "lens flare brightness {brightness} out of range 10..=300"
        )));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let cx = center.0.clamp(0.0, 1.0) * (w as f64 - 1.0);
    let cy = center.1.clamp(0.0, 1.0) * (h as f64 - 1.0);
    let (spokes, perp, ghosts) = lens_model(lens);
    let scale = 0.5 * (w as f64).hypot(h as f64);
    let gain = brightness / 100.0;
    let core_r = 0.18 * scale;
    let mirror_x = w as f64 - 1.0 - cx;
    let mirror_y = h as f64 - 1.0 - cy;
    let ghost_pos: Vec<(f64, f64, f64, f64)> = ghosts
        .iter()
        .map(|g| {
            (
                cx + g.along * (mirror_x - cx),
                cy + g.along * (mirror_y - cy) + perp * h as f64,
                g.radius * scale,
                g.alpha,
            )
        })
        .collect();
    let ray_len = 0.9 * scale;
    let ray_amp = 0.6;
    let ray_sigma = 0.07;
    let streak_amp = 0.35;
    let streak_h = (0.015 * h as f64).max(1.0);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let dist = dx.hypot(dy);
            let mut add = (-(dist / core_r).powi(2)).exp();
            let mut ray = 0.0_f64;
            let ang = dy.atan2(dx);
            for k in 0..spokes {
                let phi = k as f64 * std::f64::consts::TAU / spokes as f64;
                let mut d = ang - phi;
                if d > std::f64::consts::PI {
                    d -= std::f64::consts::TAU;
                } else if d < -std::f64::consts::PI {
                    d += std::f64::consts::TAU;
                }
                ray = ray.max((-(d / ray_sigma).powi(2)).exp());
            }
            add += ray_amp * ray * (-(dist / ray_len).powi(2)).exp();
            for &(gx, gy, gr, galpha) in &ghost_pos {
                let gd = (x as f64 - gx).hypot(y as f64 - gy);
                add += galpha * (-(gd / gr).powi(2)).exp();
            }
            if lens == LensType::MoviePrime {
                add += streak_amp
                    * (-(dy / streak_h).powi(2)).exp()
                    * (-(dx / (0.7 * w as f64)).powi(2)).exp();
            }
            for c in 0..planes {
                let idx = c * n + y * w + x;
                let v = buf.data[idx] as f64 + add * 255.0 * gain;
                buf.data[idx] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

/// The three lamps CS6's Lighting Effects offers, in the order its Light Type
/// list gives them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LightType {
    #[default]
    Spot,
    Point,
    Infinite,
}

impl LightType {
    pub fn from_i32(value: i32) -> LightType {
        match value {
            1 => LightType::Point,
            2 => LightType::Infinite,
            _ => LightType::Spot,
        }
    }
}

/// Which channel is read as a height map — CS6's Texture list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureChannel {
    #[default]
    None,
    Red,
    Green,
    Blue,
}

impl TextureChannel {
    pub fn from_i32(value: i32) -> TextureChannel {
        match value {
            1 => TextureChannel::Red,
            2 => TextureChannel::Green,
            3 => TextureChannel::Blue,
            _ => TextureChannel::None,
        }
    }
}

/// Everything CS6's Lighting Effects Properties panel collects, for one lamp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lighting {
    pub kind: LightType,
    pub color: [u8; 3],
    pub intensity: f32,
    pub hotspot: f32,
    pub colorize: [u8; 3],
    pub ambience: f32,
    pub exposure: f32,
    pub gloss: f32,
    pub metallic: f32,
    pub texture: TextureChannel,
    pub height: f32,
    pub center: (f32, f32),
    pub size: f32,
    pub angle: f32,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            kind: LightType::Spot,
            color: [255, 255, 255],
            intensity: 25.0,
            hotspot: 44.0,
            colorize: [255, 255, 255],
            ambience: 0.0,
            exposure: 0.0,
            gloss: 0.0,
            metallic: 0.0,
            texture: TextureChannel::None,
            height: 50.0,
            center: (0.5, 0.5),
            size: 0.45,
            angle: 45.0,
        }
    }
}

fn dot3(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

fn normalize3(v: (f32, f32, f32)) -> (f32, f32, f32) {
    let length = (v.0 * v.0 + v.1 * v.1 + v.2 * v.2).sqrt();
    if length <= f32::EPSILON {
        return (0.0, 0.0, 1.0);
    }
    (v.0 / length, v.1 / length, v.2 / length)
}

/// Light the picture as though a lamp were shining on it — Filter ▸ Render ▸
/// Lighting Effects.
///
/// The picture is treated as a surface and re-lit: ambient, diffuse by `N·L`,
/// and a Blinn specular highlight, with the normal read off whichever channel
/// the Texture list names. CS6's closed workspace (a Lights panel, on-canvas
/// handles, presets) is out of scope; this is one lamp set from a dialog.
pub fn lighting_effects(buf: &mut PixelBuffer, opt: &Lighting) -> Result<(), FilterError> {
    let n = validate(buf)?;
    for (name, value) in [
        ("intensity", opt.intensity),
        ("hotspot", opt.hotspot),
        ("ambience", opt.ambience),
        ("exposure", opt.exposure),
        ("gloss", opt.gloss),
        ("metallic", opt.metallic),
    ] {
        if !value.is_finite() || !(-100.0..=100.0).contains(&value) {
            return Err(FilterError::InvalidParams(format!(
                "lighting {name} {value} out of range -100..=100"
            )));
        }
    }
    if !opt.height.is_finite() || !(0.0..=100.0).contains(&opt.height) {
        return Err(FilterError::InvalidParams(format!(
            "lighting height {} out of range 0..=100",
            opt.height
        )));
    }
    if !opt.size.is_finite() || opt.size <= 0.0 {
        return Err(FilterError::InvalidParams(format!(
            "lighting size {} must be finite and positive",
            opt.size
        )));
    }
    if !opt.center.0.is_finite() || !opt.center.1.is_finite() || !opt.angle.is_finite() {
        return Err(FilterError::InvalidParams(
            "lighting center/angle must be finite".into(),
        ));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let (fw, fh) = (w as f32, h as f32);
    let planes = (buf.channels as usize).min(3);

    // Sizes are fractions of the half-diagonal, as with the flare, so the
    // dialog's shrunk preview matches the full render.
    let span = 0.5 * (fw * fw + fh * fh).sqrt();
    let reach = opt.size.clamp(0.02, 3.0) * span;
    let at = (opt.center.0 * fw, opt.center.1 * fh);
    let heading = opt.angle.to_radians();

    let lamp = match opt.kind {
        LightType::Spot => (
            at.0 + heading.cos() * reach * 0.55,
            at.1 + heading.sin() * reach * 0.55,
            reach * 0.85,
        ),
        LightType::Point => (at.0, at.1, reach * 0.7),
        LightType::Infinite => (at.0, at.1, reach),
    };
    let sun = {
        let elevation = 50.0f32.to_radians();
        let flat = elevation.cos();
        normalize3((
            -heading.cos() * flat,
            -heading.sin() * flat,
            elevation.sin(),
        ))
    };

    let hotspot = ((opt.hotspot.clamp(-100.0, 100.0) + 100.0) / 200.0).clamp(0.0, 0.95);
    let strength = opt.intensity.clamp(-100.0, 100.0) / 25.0;
    let ambient = opt.ambience.clamp(-100.0, 100.0) / 100.0;
    let exposure = (opt.exposure.clamp(-100.0, 100.0) / 50.0).exp2();
    let shine = (opt.gloss.clamp(-100.0, 100.0) / 100.0).max(0.0).powf(1.5);
    let tightness = 2.0f32.powf(1.0 + (opt.gloss.clamp(-100.0, 100.0) + 100.0) / 200.0 * 6.0);
    let metal = ((opt.metallic.clamp(-100.0, 100.0) + 100.0) / 200.0).clamp(0.0, 1.0);

    let lamp_color = (
        opt.color[0] as f32 / 255.0,
        opt.color[1] as f32 / 255.0,
        opt.color[2] as f32 / 255.0,
    );
    let fill_color = (
        opt.colorize[0] as f32 / 255.0,
        opt.colorize[1] as f32 / 255.0,
        opt.colorize[2] as f32 / 255.0,
    );

    // The height map, read before anything is written: a normal needs the
    // neighbours of the pixel it belongs to, and those are about to change.
    let relief = (opt.texture != TextureChannel::None).then(|| {
        let channel = match opt.texture {
            TextureChannel::Red => 0,
            TextureChannel::Green => 1,
            _ => 2,
        };
        (0..n)
            .map(|i| buf.data[channel * n + i] as f32 / 255.0)
            .collect::<Vec<f32>>()
    });
    let relief_scale = opt.height.clamp(0.0, 100.0) / 100.0 * span * 0.05;

    for y in 0..h {
        let py = y as f32 + 0.5;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let normal = match &relief {
                None => (0.0, 0.0, 1.0),
                Some(map) => {
                    let sample = |sx: usize, sy: usize| map[sy * w + sx];
                    let (left, right) = (x.saturating_sub(1), (x + 1).min(w - 1));
                    let (up, down) = (y.saturating_sub(1), (y + 1).min(h - 1));
                    normalize3((
                        (sample(left, y) - sample(right, y)) * relief_scale,
                        (sample(x, up) - sample(x, down)) * relief_scale,
                        1.0,
                    ))
                }
            };

            let (to_lamp, falloff) = match opt.kind {
                LightType::Infinite => (sun, 1.0),
                LightType::Point => {
                    let away = ((px - at.0).powi(2) + (py - at.1).powi(2)).sqrt() / reach;
                    (
                        normalize3((lamp.0 - px, lamp.1 - py, lamp.2)),
                        (1.0 - away * away).max(0.0),
                    )
                }
                LightType::Spot => {
                    let away = ((px - at.0).powi(2) + (py - at.1).powi(2)).sqrt() / reach;
                    let edge = if away <= hotspot {
                        1.0
                    } else {
                        let t = ((1.0 - away) / (1.0 - hotspot)).clamp(0.0, 1.0);
                        t * t * (3.0 - 2.0 * t)
                    };
                    (normalize3((lamp.0 - px, lamp.1 - py, lamp.2)), edge)
                }
            };

            let facing = dot3(normal, to_lamp).max(0.0);
            let diffuse = facing * falloff * strength;
            let highlight = if shine > 0.0 && falloff > 0.0 {
                let half = normalize3((to_lamp.0, to_lamp.1, to_lamp.2 + 1.0));
                dot3(normal, half).max(0.0).powf(tightness) * falloff * shine * strength.abs()
            } else {
                0.0
            };

            let i = y * w + x;
            let surface = [
                buf.data[i] as f32 / 255.0,
                buf.data[n + i] as f32 / 255.0,
                buf.data[2 * n + i] as f32 / 255.0,
            ];
            for (c, &own) in surface.iter().enumerate().take(planes) {
                let light = match c {
                    0 => fill_color.0 * ambient + lamp_color.0 * diffuse,
                    1 => fill_color.1 * ambient + lamp_color.1 * diffuse,
                    _ => fill_color.2 * ambient + lamp_color.2 * diffuse,
                };
                let lamp_channel = match c {
                    0 => lamp_color.0,
                    1 => lamp_color.1,
                    _ => lamp_color.2,
                };
                let spec = highlight * (lamp_channel * (1.0 - metal) + own * metal);
                let value = (own * light + spec) * exposure;
                buf.data[c * n + i] = (value * 255.0).clamp(0.0, 255.0) as u8;
            }
            // Alpha stands: relighting a layer does not change its shape.
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::luma::luma;

    fn plane(w: u32, h: u32, fill: [u8; 3]) -> PixelBuffer {
        let n = w as usize * h as usize;
        let mut data = vec![0u8; n * 4];
        for p in 0..n {
            for (c, &v) in fill.iter().enumerate() {
                data[c * n + p] = v;
            }
            data[3 * n + p] = 200;
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: 4,
            data: data.into(),
        }
    }

    /// Deterministic non-uniform pattern so an additive/replacing filter
    /// cannot be invariant.
    fn pattern(w: u32, h: u32) -> PixelBuffer {
        let n = w as usize * h as usize;
        let mut data = vec![0u8; n * 4];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let p = y * w as usize + x;
                data[p] = (x * 37 + y * 17) as u8;
                data[n + p] = (x * 11 + y * 53) as u8;
                data[2 * n + p] = (x * 71 + y * 5) as u8;
                data[3 * n + p] = 200;
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: 4,
            data: data.into(),
        }
    }

    fn plane_std(buf: &PixelBuffer, plane: usize) -> f64 {
        let n = buf.pixel_count();
        let vals = &buf.data[plane * n..plane * n + n];
        let mean = vals.iter().map(|&v| v as f64).sum::<f64>() / n as f64;
        let var = vals.iter().map(|&v| (v as f64 - mean).powi(2)).sum::<f64>() / n as f64;
        var.sqrt()
    }

    fn argmax_pixel(buf: &PixelBuffer) -> (usize, usize) {
        let n = buf.pixel_count();
        let w = buf.width as usize;
        let (mut best, mut best_l) = (0usize, f64::MIN);
        for p in 0..n {
            let l = luma(
                buf.data[p] as f64,
                buf.data[n + p] as f64,
                buf.data[2 * n + p] as f64,
            );
            if l > best_l {
                best_l = l;
                best = p;
            }
        }
        (best % w, best / w)
    }

    fn assert_channels_within(buf: &PixelBuffer, a: [u8; 3], b: [u8; 3]) {
        let n = buf.pixel_count();
        for p in 0..n {
            for c in 0..3 {
                let v = buf.data[c * n + p];
                assert!(
                    (a[c].min(b[c])..=a[c].max(b[c])).contains(&v),
                    "channel {c} pixel {p} value {v} outside [{}, {}]",
                    a[c].min(b[c]),
                    a[c].max(b[c])
                );
            }
        }
    }

    #[test]
    fn clouds_same_seed_is_bit_identical_and_different_seed_differs() {
        let base = plane(32, 32, [0, 0, 0]);
        let mut a = base.clone();
        let mut b = base.clone();
        let mut c = base.clone();
        clouds(&mut a, [10, 20, 30], [200, 210, 220], false, 42).unwrap();
        clouds(&mut b, [10, 20, 30], [200, 210, 220], false, 42).unwrap();
        clouds(&mut c, [10, 20, 30], [200, 210, 220], false, 43).unwrap();
        assert_eq!(a.data, b.data);
        assert_ne!(a.data, c.data);
    }

    #[test]
    fn clouds_rgb_stays_within_color_endpoints_and_alpha_untouched() {
        let a = [10, 200, 30];
        let b = [240, 20, 90];
        let mut buf = plane(24, 24, [7, 7, 7]);
        clouds(&mut buf, a, b, false, 5).unwrap();
        assert_channels_within(&buf, a, b);
        let n = buf.pixel_count();
        assert!(buf.data[3 * n..].iter().all(|&v| v == 200));
    }

    #[test]
    fn starker_clouds_differ_and_raise_contrast() {
        let base = plane(32, 32, [0, 0, 0]);
        let mut plain = base.clone();
        let mut stark = base.clone();
        clouds(&mut plain, [0, 0, 0], [255, 255, 255], false, 9).unwrap();
        clouds(&mut stark, [0, 0, 0], [255, 255, 255], true, 9).unwrap();
        assert_ne!(plain.data, stark.data);
        for c in 0..3 {
            assert!(
                plane_std(&stark, c) > plane_std(&plain, c),
                "channel {c}: starker std should exceed plain"
            );
        }
    }

    #[test]
    fn difference_clouds_equals_abs_difference_with_clouds() {
        let mut orig = pattern(16, 16);
        let mut cloud_ref = orig.clone();
        clouds(&mut cloud_ref, [30, 40, 50], [220, 180, 90], true, 11).unwrap();
        let before = orig.data.clone();
        difference_clouds(&mut orig, [30, 40, 50], [220, 180, 90], true, 11).unwrap();
        let n = orig.pixel_count();
        for p in 0..n {
            for c in 0..3 {
                let expected = before[c * n + p].abs_diff(cloud_ref.data[c * n + p]);
                assert_eq!(orig.data[c * n + p], expected, "plane {c} pixel {p}");
            }
            assert_eq!(orig.data[3 * n + p], 200);
        }
    }

    #[test]
    fn second_difference_clouds_application_differs() {
        let mut buf = pattern(16, 16);
        difference_clouds(&mut buf, [0, 0, 0], [255, 255, 255], false, 3).unwrap();
        let first = buf.data.clone();
        difference_clouds(&mut buf, [0, 0, 0], [255, 255, 255], false, 3).unwrap();
        assert_ne!(buf.data, first);
    }

    #[test]
    fn fibers_same_seed_is_deterministic_and_rgb_within_endpoints() {
        let a = [200, 150, 100];
        let b = [250, 240, 230];
        let base = plane(32, 32, [0, 0, 0]);
        let mut x = base.clone();
        let mut y = base.clone();
        fibers(&mut x, 16.0, 4.0, a, b, 7).unwrap();
        fibers(&mut y, 16.0, 4.0, a, b, 7).unwrap();
        assert_eq!(x.data, y.data);
        assert_channels_within(&x, a, b);
        let n = x.pixel_count();
        assert!(x.data[3 * n..].iter().all(|&v| v == 200));
    }

    #[test]
    fn fibers_variance_grows_along_x_variation() {
        let base = plane(64, 32, [0, 0, 0]);
        let mut low = base.clone();
        let mut high = base.clone();
        fibers(&mut low, 5.0, 4.0, [0, 0, 0], [255, 255, 255], 12).unwrap();
        fibers(&mut high, 95.0, 4.0, [0, 0, 0], [255, 255, 255], 12).unwrap();
        let row_delta = |buf: &PixelBuffer| -> f64 {
            let w = buf.width as usize;
            let h = buf.height as usize;
            let mut total = 0.0;
            for y in 0..h {
                let mut row = 0.0;
                for x in 1..w {
                    row += buf.data[y * w + x].abs_diff(buf.data[y * w + x - 1]) as f64;
                }
                total += row / (w - 1) as f64;
            }
            total / h as f64
        };
        let low_d = row_delta(&low);
        let high_d = row_delta(&high);
        assert!(
            low_d <= high_d,
            "low-variance along-x delta {low_d} should not exceed high {high_d}"
        );
    }

    #[test]
    fn fibers_params_are_validated_and_buffer_untouched() {
        let base = plane(8, 8, [90, 60, 30]);
        for &(variance, strength) in &[
            (-1.0, 4.0),
            (101.0, 4.0),
            (16.0, 0.0),
            (16.0, 101.0),
            (f64::NAN, 4.0),
            (16.0, f64::NAN),
        ] {
            let mut buf = base.clone();
            let err =
                fibers(&mut buf, variance, strength, [0, 0, 0], [255, 255, 255], 1).unwrap_err();
            assert!(
                matches!(err, FilterError::InvalidParams(_)),
                "variance {variance} strength {strength} should be rejected"
            );
            assert_eq!(buf.data, base.data);
        }
    }

    #[test]
    fn lens_flare_is_deterministic_and_lens_types_differ() {
        let base = plane(32, 32, [0, 0, 0]);
        let mut a = base.clone();
        let mut b = base.clone();
        lens_flare(&mut a, 100.0, (0.5, 0.5), LensType::Zoom).unwrap();
        lens_flare(&mut b, 100.0, (0.5, 0.5), LensType::Zoom).unwrap();
        assert_eq!(a.data, b.data);

        let mut out_of_range = base.clone();
        lens_flare(&mut out_of_range, 100.0, (-3.0, 5.0), LensType::Zoom).unwrap();
        let n = out_of_range.pixel_count();
        assert!(out_of_range.data[3 * n..].iter().all(|&v| v == 200));

        let mut outputs = Vec::new();
        for lens in [
            LensType::Zoom,
            LensType::Prime35,
            LensType::Prime105,
            LensType::MoviePrime,
        ] {
            let mut buf = base.clone();
            lens_flare(&mut buf, 100.0, (0.5, 0.5), lens).unwrap();
            outputs.push(buf.data);
        }
        for i in 0..outputs.len() {
            for j in i + 1..outputs.len() {
                assert_ne!(outputs[i], outputs[j], "lens types {i} and {j} collided");
            }
        }
    }

    #[test]
    fn lens_flare_brightness_is_monotonic_at_core() {
        let base = plane(32, 32, [0, 0, 0]);
        let core = |brightness: f64| -> f64 {
            let mut buf = base.clone();
            lens_flare(&mut buf, brightness, (0.5, 0.5), LensType::Prime35).unwrap();
            let n = buf.pixel_count();
            let p = (buf.height as usize / 2) * buf.width as usize + buf.width as usize / 2;
            luma(
                buf.data[p] as f64,
                buf.data[n + p] as f64,
                buf.data[2 * n + p] as f64,
            )
        };
        assert!(core(10.0) <= core(100.0));
        assert!(core(100.0) <= core(300.0));
    }

    #[test]
    fn lens_flare_argmax_moves_toward_mapped_center() {
        let base = plane(64, 64, [0, 0, 0]);
        let mut left = base.clone();
        let mut right = base.clone();
        lens_flare(&mut left, 10.0, (0.2, 0.5), LensType::Zoom).unwrap();
        lens_flare(&mut right, 10.0, (0.8, 0.5), LensType::Zoom).unwrap();
        let (lx, ly) = argmax_pixel(&left);
        let (rx, ry) = argmax_pixel(&right);
        let mapped = |c: f64| c.clamp(0.0, 1.0) * 63.0;
        let dist = |x: usize, y: usize, cx: f64, cy: f64| {
            ((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt()
        };
        assert!(
            dist(lx, ly, mapped(0.2), mapped(0.5)) < dist(lx, ly, mapped(0.8), mapped(0.5)),
            "left flare argmax ({lx}, {ly}) should sit near the mapped left center"
        );
        assert!(
            dist(rx, ry, mapped(0.8), mapped(0.5)) < dist(rx, ry, mapped(0.2), mapped(0.5)),
            "right flare argmax ({rx}, {ry}) should sit near the mapped right center"
        );
    }

    #[test]
    fn lens_flare_clamps_white_and_preserves_alpha() {
        let mut buf = plane(32, 32, [255, 255, 255]);
        lens_flare(&mut buf, 300.0, (0.5, 0.5), LensType::MoviePrime).unwrap();
        let n = buf.pixel_count();
        for p in 0..n {
            for c in 0..3 {
                assert_eq!(buf.data[c * n + p], 255, "saturated pixel must stay 255");
            }
            assert_eq!(buf.data[3 * n + p], 200);
        }
    }

    #[test]
    fn lens_flare_brightness_is_validated_and_buffer_untouched() {
        let base = plane(8, 8, [40, 50, 60]);
        for brightness in [9.0, 301.0, f64::NAN] {
            let mut buf = base.clone();
            let err = lens_flare(&mut buf, brightness, (0.5, 0.5), LensType::Zoom).unwrap_err();
            assert!(
                matches!(err, FilterError::InvalidParams(_)),
                "brightness {brightness} should be rejected"
            );
            assert_eq!(buf.data, base.data);
        }
    }

    #[test]
    fn lighting_effects_relights_and_preserves_alpha() {
        let base = pattern(32, 32);
        let before = base.data.clone();
        let mut buf = base.clone();
        lighting_effects(&mut buf, &Lighting::default()).unwrap();
        let n = buf.pixel_count();
        assert_ne!(buf.data, before, "relighting must change the colour planes");
        assert_eq!(buf.data[3 * n..], before[3 * n..], "alpha untouched");
    }

    #[test]
    fn lighting_effects_each_type_differs_and_bump_reads_channel() {
        let mut outputs = Vec::new();
        for kind in [LightType::Spot, LightType::Point, LightType::Infinite] {
            let mut buf = pattern(24, 24);
            let opt = Lighting {
                kind,
                ..Lighting::default()
            };
            lighting_effects(&mut buf, &opt).unwrap();
            outputs.push(buf.data);
        }
        for i in 0..outputs.len() {
            for j in i + 1..outputs.len() {
                assert_ne!(outputs[i], outputs[j], "light types {i} and {j} collided");
            }
        }

        let mut flat = pattern(16, 16);
        let mut textured = flat.clone();
        let textured_opt = Lighting {
            texture: TextureChannel::Green,
            height: 100.0,
            ..Lighting::default()
        };
        lighting_effects(&mut textured, &textured_opt).unwrap();
        let flat_opt = Lighting {
            texture: TextureChannel::None,
            ..textured_opt
        };
        lighting_effects(&mut flat, &flat_opt).unwrap();
        assert_ne!(flat.data, textured.data);
    }

    #[test]
    fn lighting_effects_rejects_bad_params_and_leaves_buffer_untouched() {
        let base = pattern(8, 8);
        for opt in [
            Lighting {
                intensity: 101.0,
                ..Lighting::default()
            },
            Lighting {
                height: -1.0,
                ..Lighting::default()
            },
            Lighting {
                size: 0.0,
                ..Lighting::default()
            },
            Lighting {
                angle: f32::NAN,
                ..Lighting::default()
            },
        ] {
            let mut buf = base.clone();
            let err = lighting_effects(&mut buf, &opt).unwrap_err();
            assert!(matches!(err, FilterError::InvalidParams(_)));
            assert_eq!(buf.data, base.data);
        }
    }
}
