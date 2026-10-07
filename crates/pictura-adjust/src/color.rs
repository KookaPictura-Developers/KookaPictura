use pictura_core::PixelBuffer;

use crate::common::{hsl_to_rgb, luma, planes_mut, rgb_to_hsl};
use crate::types::{
    AdjustError, BlackWhiteParams, ChannelMixerParams, ColorBalanceParams, HueSaturationParams,
    PhotoFilterParams, SelectiveColorMethod, SelectiveColorParams, SelectiveRange, VibranceParams,
};

// ---------------------------------------------------------------------------
// Colour adjustments
// ---------------------------------------------------------------------------

/// Reject out-of-range Master or range settings.
pub(crate) fn validate_hue_saturation(p: &HueSaturationParams) -> Result<(), AdjustError> {
    let triples = std::iter::once((p.hue, p.saturation, p.lightness))
        .chain(p.ranges.iter().map(|r| (r.hue, r.saturation, r.lightness)));
    for (hue, saturation, lightness) in triples {
        if !(-180..=180).contains(&hue) {
            return Err(AdjustError::InvalidParams("hue must be -180..=180".into()));
        }
        if !(-100..=100).contains(&saturation) {
            return Err(AdjustError::InvalidParams(
                "saturation must be -100..=100".into(),
            ));
        }
        if !(-100..=100).contains(&lightness) {
            return Err(AdjustError::InvalidParams(
                "lightness must be -100..=100".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn hue_saturation_is_identity(p: &HueSaturationParams) -> bool {
    std::iter::once((p.hue, p.saturation, p.lightness))
        .chain(p.ranges.iter().map(|r| (r.hue, r.saturation, r.lightness)))
        .all(|t| t == (0, 0, 0))
}

/// One pixel (`0.0..=1.0` RGB) through Hue/Saturation. Each colour range adds
/// its settings scaled by its weight at the pixel's hue.
/// ponytail: the range blend (additive, faded out toward gray by chroma so a
/// neutral is not read as red) approximates Photoshop's unpublished one; no
/// baseline carries a range edit.
pub(crate) fn hue_saturation_rgb(p: &HueSaturationParams, r: f64, g: f64, b: f64) -> [f64; 3] {
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let mut dh = p.hue as f64;
    let mut ds = p.saturation as f64 / 100.0;
    let mut dl = p.lightness as f64 / 100.0;
    if !p.ranges.is_empty() {
        let presence = ((r.max(g).max(b) - r.min(g).min(b)) * 4.0).min(1.0);
        for range in &p.ranges {
            let w = range.weight(h) * presence;
            dh += w * range.hue as f64;
            ds += w * range.saturation as f64 / 100.0;
            dl += w * range.lightness as f64 / 100.0;
        }
        ds = ds.clamp(-1.0, 1.0);
        dl = dl.clamp(-1.0, 1.0);
    }
    let h = (h + dh).rem_euclid(360.0);
    let s = (s * (1.0 + ds)).clamp(0.0, 1.0);
    let (nr, ng, nb) = hsl_to_rgb(h, s, l);
    // Lightness blends each channel toward white or black after the hue and
    // saturation move, as Photoshop does: scaling HSL L instead keeps S and
    // turns near-neutral dark noise into saturated patches.
    let lighten = |c: f64| {
        if dl >= 0.0 {
            c + dl * (1.0 - c)
        } else {
            c * (1.0 + dl)
        }
        .clamp(0.0, 1.0)
    };
    [lighten(nr), lighten(ng), lighten(nb)]
}

pub(crate) fn hue_saturation(
    p: &HueSaturationParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    validate_hue_saturation(p)?;
    if hue_saturation_is_identity(p) {
        return Ok(());
    }
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let [nr, ng, nb] = hue_saturation_rgb(
            p,
            *rv as f64 / 255.0,
            *gv as f64 / 255.0,
            *bv as f64 / 255.0,
        );
        *rv = (nr * 255.0).round().clamp(0.0, 255.0) as u8;
        *gv = (ng * 255.0).round().clamp(0.0, 255.0) as u8;
        *bv = (nb * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    Ok(())
}

pub(crate) fn vibrance(
    p: &VibranceParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    if !(-100..=100).contains(&p.vibrance) {
        return Err(AdjustError::InvalidParams(
            "vibrance must be -100..=100".into(),
        ));
    }
    if !(-100..=100).contains(&p.saturation) {
        return Err(AdjustError::InvalidParams(
            "saturation must be -100..=100".into(),
        ));
    }
    if p.vibrance == 0 && p.saturation == 0 {
        return Ok(());
    }
    let kv = p.vibrance as f64 / 100.0;
    let ks = p.saturation as f64 / 100.0;
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (h, s, l) = rgb_to_hsl(*rv as f64 / 255.0, *gv as f64 / 255.0, *bv as f64 / 255.0);
        // Approximation: boost falls off as (1 - S) (the spec's `S*(1-S)` text
        // contradicts its own acceptance criteria) and is damped in the skin
        // hue band. The exact falloff/skin model is closed.
        let keep = 1.0 - 0.5 * skin_bump(h);
        let s = (s + kv * (1.0 - s) * keep).clamp(0.0, 1.0);
        let s = (s * (1.0 + ks)).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        *rv = (nr * 255.0).round().clamp(0.0, 255.0) as u8;
        *gv = (ng * 255.0).round().clamp(0.0, 255.0) as u8;
        *bv = (nb * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    Ok(())
}

/// Smooth bump over the orange/red skin band (~30° centre, 30° half-width).
pub(crate) fn skin_bump(h: f64) -> f64 {
    let d = ((h - 30.0 + 540.0).rem_euclid(360.0) - 180.0).abs();
    if d >= 30.0 {
        0.0
    } else {
        0.5 * (1.0 + (std::f64::consts::PI * d / 30.0).cos())
    }
}

pub(crate) fn color_balance(
    p: &ColorBalanceParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    for band in [&p.shadows, &p.midtones, &p.highlights] {
        if band.iter().any(|v| !v.is_finite() || v.abs() > 100.0) {
            return Err(AdjustError::InvalidParams(
                "color balance values must be -100..=100".into(),
            ));
        }
    }
    if p.shadows == [0.0; 3] && p.midtones == [0.0; 3] && p.highlights == [0.0; 3] {
        return Ok(());
    }
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
        let y = luma(ir, ig, ib) / 255.0;
        // Overlapping parabola windows: shadow=1 at Y=0, highlight=1 at Y=1,
        // midtone peaks at Y=0.5. The exact falloffs are closed.
        let sh = (1.0 - y) * (1.0 - y);
        let hi = y * y;
        let mid = (1.0 - sh - hi).max(0.0);
        let w = [sh, mid, hi];
        let mut out = [ir, ig, ib];
        for (c, o) in out.iter_mut().enumerate() {
            let delta = w[0] * p.shadows[c] + w[1] * p.midtones[c] + w[2] * p.highlights[c];
            *o = (*o + delta).clamp(0.0, 255.0);
        }
        if p.preserve_luminosity {
            let y1 = luma(out[0], out[1], out[2]);
            if y1 > 0.0 {
                let scale = luma(ir, ig, ib) / y1;
                for o in &mut out {
                    *o = (*o * scale).clamp(0.0, 255.0);
                }
            }
        }
        *rv = out[0].round() as u8;
        *gv = out[1].round() as u8;
        *bv = out[2].round() as u8;
    }
    Ok(())
}

pub(crate) fn black_white(
    p: &BlackWhiteParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    let weights = [p.red, p.yellow, p.green, p.cyan, p.blue, p.magenta];
    if weights
        .iter()
        .any(|v| !v.is_finite() || !(-200.0..=300.0).contains(v))
    {
        return Err(AdjustError::InvalidParams(
            "black & white weights must be -200..=300 percent".into(),
        ));
    }
    if p.tint && p.tint_color == [0, 0, 0] {
        // A black tint is a no-op; avoid tinting everything to black.
        return Err(AdjustError::InvalidParams(
            "tint colour must not be black".into(),
        ));
    }
    let (rw, yw, gw, cw, bw, mw) = (
        p.red / 100.0,
        p.yellow / 100.0,
        p.green / 100.0,
        p.cyan / 100.0,
        p.blue / 100.0,
        p.magenta / 100.0,
    );
    let tint_hsl = if p.tint {
        let (h, s, _) = rgb_to_hsl(
            p.tint_color[0] as f64 / 255.0,
            p.tint_color[1] as f64 / 255.0,
            p.tint_color[2] as f64 / 255.0,
        );
        Some((h, s))
    } else {
        None
    };
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (mut cr, mut cg, mut cb) = (*rv as f64, *gv as f64, *bv as f64);
        let neutral = cr.min(cg).min(cb);
        cr -= neutral;
        cg -= neutral;
        cb -= neutral;
        let mut gray = neutral;
        if cr == 0.0 {
            let cyan = cg.min(cb);
            cg -= cyan;
            cb -= cyan;
            gray += cyan * cw + cg * gw + cb * bw;
        } else if cg == 0.0 {
            let magenta = cr.min(cb);
            cr -= magenta;
            cb -= magenta;
            gray += magenta * mw + cr * rw + cb * bw;
        } else {
            let yellow = cr.min(cg);
            cr -= yellow;
            cg -= yellow;
            gray += yellow * yw + cr * rw + cg * gw;
        }
        let out = gray.round().clamp(0.0, 255.0) as u8;
        let (nr, ng, nb) = match tint_hsl {
            Some((h, s)) => {
                let (tr, tg, tb) = hsl_to_rgb(h, s, out as f64 / 255.0);
                (
                    (tr * 255.0).round() as u8,
                    (tg * 255.0).round() as u8,
                    (tb * 255.0).round() as u8,
                )
            }
            None => (out, out, out),
        };
        *rv = nr;
        *gv = ng;
        *bv = nb;
    }
    Ok(())
}

pub(crate) fn photo_filter(
    p: &PhotoFilterParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    if !p.density.is_finite() || !(0.0..=100.0).contains(&p.density) {
        return Err(AdjustError::InvalidParams(
            "photo filter density must be 0..=100".into(),
        ));
    }
    if p.density == 0.0 {
        return Ok(());
    }
    let d = p.density / 100.0;
    let f = [p.color[0] as f64, p.color[1] as f64, p.color[2] as f64];
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
        let mut out = [
            ir * (1.0 - d) + f[0] * d,
            ig * (1.0 - d) + f[1] * d,
            ib * (1.0 - d) + f[2] * d,
        ];
        if p.preserve_luminosity {
            let y1 = luma(out[0], out[1], out[2]);
            if y1 > 0.0 {
                let scale = luma(ir, ig, ib) / y1;
                for o in &mut out {
                    *o = (*o * scale).clamp(0.0, 255.0);
                }
            }
        }
        *rv = out[0].round().clamp(0.0, 255.0) as u8;
        *gv = out[1].round().clamp(0.0, 255.0) as u8;
        *bv = out[2].round().clamp(0.0, 255.0) as u8;
    }
    Ok(())
}

pub(crate) fn channel_mixer(
    p: &ChannelMixerParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    let mixes = [&p.red, &p.green, &p.blue];
    if mixes
        .iter()
        .chain(std::iter::once(&&p.constant))
        .any(|m| m.iter().any(|v| !v.is_finite() || v.abs() > 200.0))
    {
        return Err(AdjustError::InvalidParams(
            "channel mixer weights must be -200..=200 percent".into(),
        ));
    }
    let (r, g, b) = planes_mut(buf, n);
    if p.monochrome {
        // Monochrome applies one source mix to every output; use the Red row.
        let w = p.red;
        let c = p.constant[0] / 100.0 * 255.0;
        for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
            let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
            let v = ((w[0] * ir + w[1] * ig + w[2] * ib) / 100.0 + c)
                .round()
                .clamp(0.0, 255.0) as u8;
            *rv = v;
            *gv = v;
            *bv = v;
        }
    } else {
        for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
            let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
            let mix = |w: &[f64; 3], k: f64| {
                ((w[0] * ir + w[1] * ig + w[2] * ib) / 100.0 + k / 100.0 * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            *rv = mix(&p.red, p.constant[0]);
            *gv = mix(&p.green, p.constant[1]);
            *bv = mix(&p.blue, p.constant[2]);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Selective Color (libpsd's integer RGB -> CMYK -> RGB pipeline)
// ---------------------------------------------------------------------------

/// Reject a correction outside `-100..=100`.
pub(crate) fn validate_selective_color(p: &SelectiveColorParams) -> Result<(), AdjustError> {
    if p.ranges
        .iter()
        .flat_map(|r| [r.c, r.m, r.y, r.k])
        .any(|v| !(-100..=100).contains(&v))
    {
        return Err(AdjustError::InvalidParams(
            "selective color corrections must be -100..=100".into(),
        ));
    }
    Ok(())
}

pub(crate) fn selective_color_is_identity(p: &SelectiveColorParams) -> bool {
    p.ranges.iter().all(|r| *r == SelectiveRange::default())
}

pub(crate) fn selective_color(
    p: &SelectiveColorParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    validate_selective_color(p)?;
    if selective_color_is_identity(p) {
        return Ok(());
    }
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let out = selective_color_rgb(
            p,
            [*rv as f64 / 255.0, *gv as f64 / 255.0, *bv as f64 / 255.0],
        );
        [*rv, *gv, *bv] = out.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
    }
    Ok(())
}

/// One pixel (`0.0..=1.0` RGB) through Selective Color. Each range weighs the
/// pixel by how much of that colour it holds: a primary (Reds, Greens, Blues)
/// by max − mid when its channel is the largest, a secondary (Yellows, Cyans,
/// Magentas) by mid − min when its complement is the smallest; Whites, Neutrals,
/// and Blacks by the lightness bands of min and max. So a near-gray pixel
/// takes almost nothing from a colour range. Each weighted range then moves the
/// cyan / magenta / yellow ink of red / green / blue by `(-1 - a)·k - a` (`a`
/// the ink correction, `k` the black one), scaled by the existing ink when
/// Relative and clamped to what the channel can take.
/// ponytail: the reverse-engineered model in common use for Photoshop's
/// closed Selective Color; no baseline proves parity.
pub(crate) fn selective_color_rgb(p: &SelectiveColorParams, rgb: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = rgb;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mid = r + g + b - max - min;
    let primary = |channel: f64| if channel == max { max - mid } else { 0.0 };
    let secondary = |channel: f64| if channel == min { mid - min } else { 0.0 };
    let weights = [
        primary(r),
        secondary(b),
        primary(g),
        secondary(r),
        primary(b),
        secondary(g),
        if min > 0.5 { (min - 0.5) * 2.0 } else { 0.0 },
        1.0 - ((max - 0.5).abs() + (min - 0.5).abs()),
        if max < 0.5 { (0.5 - max) * 2.0 } else { 0.0 },
    ];
    let relative = p.method == SelectiveColorMethod::Relative;
    let mut out = rgb;
    for (range, weight) in p.ranges.iter().zip(weights) {
        if weight <= 0.0 || *range == SelectiveRange::default() {
            continue;
        }
        let k = range.k as f64 / 100.0;
        for (i, ink) in [range.c, range.m, range.y].into_iter().enumerate() {
            let a = ink as f64 / 100.0;
            let v = rgb[i];
            let mut delta = (-1.0 - a) * k - a;
            if relative {
                delta *= 1.0 - v;
            }
            out[i] += delta.clamp(-v, 1.0 - v) * weight;
        }
    }
    out.map(|v| v.clamp(0.0, 1.0))
}
