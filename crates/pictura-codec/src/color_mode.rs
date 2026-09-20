//! Profile-free conversions from a PSD color mode's stored planes to planar
//! 8-bit RGB. Bitmap and Indexed are exact; CMYK and Lab are approximations of
//! Photoshop's color-managed transforms.

/// Expand a depth-1 Bitmap plane into planar RGB. Rows are `ceil(width / 8)`
/// bytes, MSB-first; a set bit is black (0) and a clear bit is white (255).
pub(crate) fn bitmap_rows_to_rgb(bits: &[u8], width: usize, height: usize) -> Vec<u8> {
    let row_bytes = width.div_ceil(8);
    let plane = width * height;
    let mut out = vec![0u8; plane * 3];
    for y in 0..height {
        for x in 0..width {
            let byte = bits.get(y * row_bytes + x / 8).copied().unwrap_or(0);
            let value = if byte & (0x80 >> (x % 8)) != 0 {
                0
            } else {
                255
            };
            let i = y * width + x;
            out[i] = value;
            out[plane + i] = value;
            out[2 * plane + i] = value;
        }
    }
    out
}

/// Replicate one 8-bit gray plane into planar RGB.
pub(crate) fn gray_to_rgb(gray: &[u8]) -> Vec<u8> {
    let plane = gray.len();
    let mut out = Vec::with_capacity(plane * 3);
    out.extend_from_slice(gray);
    out.extend_from_slice(gray);
    out.extend_from_slice(gray);
    out
}

/// Expand a single index plane through a 768-byte palette laid out as 256 red,
/// then 256 green, then 256 blue bytes.
pub(crate) fn indexed_to_rgb(indices: &[u8], palette: &[u8; 768]) -> Vec<u8> {
    let plane = indices.len();
    let mut out = vec![0u8; plane * 3];
    for (i, &index) in indices.iter().enumerate() {
        let index = index as usize;
        out[i] = palette[index];
        out[plane + i] = palette[256 + index];
        out[2 * plane + i] = palette[512 + index];
    }
    out
}

/// ponytail: profile-free approximation of Photoshop's ICC CMYK transform;
/// saturated colors differ. Stored 0 is full ink, 255 is no ink, so each RGB
/// channel is `floor(color * black / 255)`.
pub(crate) fn cmyk_to_rgb(cmyk: &[u8]) -> Vec<u8> {
    let plane = cmyk.len() / 4;
    let (c, rest) = cmyk.split_at(plane);
    let (m, rest) = rest.split_at(plane);
    let (y, k) = rest.split_at(plane);
    let mut out = vec![0u8; plane * 3];
    for i in 0..plane {
        out[i] = (c[i] as u16 * k[i] as u16 / 255) as u8;
        out[plane + i] = (m[i] as u16 * k[i] as u16 / 255) as u8;
        out[2 * plane + i] = (y[i] as u16 * k[i] as u16 / 255) as u8;
    }
    out
}

/// ponytail: approximation of a color-managed Lab->sRGB transform. This is the
/// standard CIELAB(D50) -> sRGB(D65) conversion (D50 white through the Bradford
/// D50->D65 matrix) and matches lcms2's exact, unoptimized transform within 1
/// LSB for in-gamut colors (verified over a sweep). A renderer that precomputes
/// an optimized color LUT (psd-tools / Pillow `.convert("RGB")`) can diverge by
/// up to ~20 LSB on some in-gamut colors, so that optimized path is not the
/// reference; saturated out-of-gamut colors also clip instead of gamut-mapping.
pub(crate) fn lab_to_rgb(lab: &[u8]) -> Vec<u8> {
    const M: [[f64; 3]; 3] = [
        [3.1338561, -1.6168667, -0.4906146],
        [-0.9787684, 1.9161415, 0.0334540],
        [0.0719453, -0.2289914, 1.4052427],
    ];
    const XN: f64 = 0.96422;
    const ZN: f64 = 0.82521;
    let plane = lab.len() / 3;
    let (l, rest) = lab.split_at(plane);
    let (a, b) = rest.split_at(plane);
    let mut out = vec![0u8; plane * 3];
    for i in 0..plane {
        let fy = (l[i] as f64 * 100.0 / 255.0 + 16.0) / 116.0;
        let fx = fy + (a[i] as f64 - 128.0) / 500.0;
        let fz = fy - (b[i] as f64 - 128.0) / 200.0;
        let xyz = [XN * lab_f(fx), lab_f(fy), ZN * lab_f(fz)];
        for (c, row) in M.iter().enumerate() {
            let linear = row[0] * xyz[0] + row[1] * xyz[1] + row[2] * xyz[2];
            out[c * plane + i] = srgb_encode(linear);
        }
    }
    out
}

fn lab_f(t: f64) -> f64 {
    const EPSILON: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;
    let t3 = t * t * t;
    if t3 > EPSILON {
        t3
    } else {
        (116.0 * t - 16.0) / KAPPA
    }
}

fn srgb_encode(linear: f64) -> u8 {
    let value = if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (value.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_row_aa_is_alternating_black_white() {
        let rgb = bitmap_rows_to_rgb(&[0xAA], 8, 1);
        assert_eq!(&rgb[0..8], &[0, 255, 0, 255, 0, 255, 0, 255]);
        assert_eq!(&rgb[8..16], &rgb[0..8]);
        assert_eq!(&rgb[16..24], &rgb[0..8]);
    }

    #[test]
    fn bitmap_non_multiple_of_eight_width_pads_rows() {
        // 10px wide: 2 row bytes, the unused low 6 bits are ignored.
        let rgb = bitmap_rows_to_rgb(&[0b1010_0000, 0b1100_0000], 10, 1);
        let got: Vec<u8> = rgb[0..10].to_vec();
        assert_eq!(got, vec![0, 255, 0, 255, 255, 255, 255, 255, 0, 0]);
    }

    #[test]
    fn indexed_lookup_uses_non_interleaved_palette() {
        let mut palette = [0u8; 768];
        palette[5] = 10;
        palette[256 + 5] = 20;
        palette[512 + 5] = 30;
        let rgb = indexed_to_rgb(&[5, 0], &palette);
        assert_eq!(&rgb[0..2], &[10, 0]);
        assert_eq!(&rgb[2..4], &[20, 0]);
        assert_eq!(&rgb[4..6], &[30, 0]);
    }

    #[test]
    fn cmyk_known_and_black_white_values() {
        let rgb = cmyk_to_rgb(&[128, 64, 32, 200]);
        assert_eq!(&rgb[0..1], &[100]);
        assert_eq!(&rgb[1..2], &[50]);
        assert_eq!(&rgb[2..3], &[25]);
        assert_eq!(cmyk_to_rgb(&[0, 0, 0, 0]), vec![0, 0, 0]);
        assert_eq!(cmyk_to_rgb(&[255, 255, 255, 255]), vec![255, 255, 255]);
    }

    #[test]
    fn lab_neutral_gray_and_white_within_two() {
        let gray = lab_to_rgb(&[200, 128, 128]);
        for value in gray {
            assert!((value as i32 - 194).abs() <= 2, "gray {value}");
        }
        let white = lab_to_rgb(&[255, 128, 128]);
        for value in white {
            assert!((value as i32 - 255).abs() <= 2, "white {value}");
        }
    }

    #[test]
    fn lab_non_neutral_matches_lcms2_exact() {
        // Reference values from lcms2's exact (unoptimized) LAB->sRGB transform,
        // which this formula implements. psd-tools/Pillow `.convert("RGB")`
        // applies the optimized LUT and is deliberately not the reference.
        for (lab, expected) in [
            ([225u8, 82, 114], [60u8, 246, 246]),
            ([200, 110, 100], [123, 205, 245]),
            ([150, 110, 110], [84, 152, 172]),
            ([120, 160, 140], [164, 89, 93]),
            ([90, 120, 160], [82, 86, 28]),
            ([160, 90, 120], [17, 170, 165]),
            ([110, 100, 160], [64, 114, 45]),
            ([200, 140, 160], [230, 185, 135]),
        ] {
            let rgb = lab_to_rgb(&lab);
            for c in 0..3 {
                assert_eq!(rgb[c], expected[c], "lab {lab:?} channel {c}");
            }
        }
    }
}
