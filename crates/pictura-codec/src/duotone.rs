//! Duotone Options (the color-mode-data section of a Duotone PSD).
//!
//! `psd-tools` calls this block undocumented and preserves it; the
//! published PSD format definition documents it as **Duotone Options**, and
//! `psdparse` (`duotone.c`) and `EmilDohne/PhotoshopAPI` decode the identical
//! 524-byte layout. This module only decodes it — the composite still opens as
//! grayscale-normalized RGB because the reference's Duotone *edit* model is
//! single-channel grayscale and the multi-ink compositing algorithm is
//! unpublished.

/// A `ColorStruct`: a 2-byte color space plus four 2-byte components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InkColor {
    pub space: u16,
    pub components: [u16; 4],
}

/// One ink plate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuotoneInk {
    pub color: InkColor,
    /// Pascal-string ink name (length byte + characters).
    pub name: String,
    /// Thirteen transfer-curve points; `-1` is an unset point, otherwise the
    /// value is tenths of a percent (`1000` = 100%).
    pub transfer: [i16; 13],
    /// The two-byte override flag that follows the curve.
    pub override_value: i16,
}

/// A decoded Duotone Options block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuotoneSpec {
    pub version: u16,
    /// Number of ink plates, 1..=4 (monotone/duotone/tritone/quadtone).
    pub plates: u16,
    /// One entry per active plate, in order.
    pub inks: Vec<DuotoneInk>,
    pub dot_gain: u16,
    /// Defined overprint colors: 0, 1, 4, or 11 for 1/2/3/4 plates.
    pub overprints: Vec<InkColor>,
}

/// Size of the Duotone Options block (after the section length), per the
/// spec and `psdparse`: `4*(10+64+28) + 2 + 11*10`.
pub const DUOTONE_DATA_SIZE: usize = 4 * (10 + 64 + 28) + 2 + 11 * 10;

/// Overprint-color count for 1/2/3/4 plates.
const OVERPRINTS_PER_PLATES: [usize; 4] = [0, 1, 4, 11];

fn u16_at(data: &[u8], off: usize) -> u16 {
    u16::from_be_bytes([data[off], data[off + 1]])
}

fn i16_at(data: &[u8], off: usize) -> i16 {
    u16_at(data, off) as i16
}

fn color_at(data: &[u8], off: usize) -> InkColor {
    InkColor {
        space: u16_at(data, off),
        components: [
            u16_at(data, off + 2),
            u16_at(data, off + 4),
            u16_at(data, off + 6),
            u16_at(data, off + 8),
        ],
    }
}

/// Decode a Duotone color-mode-data block.
///
/// Returns `None` when the block is shorter than [`DUOTONE_DATA_SIZE`] or the
/// plate count is not 1..=4. Extra trailing bytes are ignored.
pub fn parse_duotone(data: &[u8]) -> Option<DuotoneSpec> {
    if data.len() < DUOTONE_DATA_SIZE {
        return None;
    }
    let version = u16_at(data, 0);
    let plates = u16_at(data, 2);
    if !(1..=4).contains(&plates) {
        return None;
    }
    let plates_usize = plates as usize;
    let inks = (0..plates_usize)
        .map(|i| {
            let name_off = 44 + i * 64;
            let len = data[name_off] as usize;
            let name_end = (name_off + 1 + len).min(name_off + 64);
            let name = String::from_utf8_lossy(&data[name_off + 1..name_end]).into_owned();
            let curve_off = 300 + i * 28;
            let mut transfer = [0i16; 13];
            for (k, point) in transfer.iter_mut().enumerate() {
                *point = i16_at(data, curve_off + k * 2);
            }
            DuotoneInk {
                color: color_at(data, 4 + i * 10),
                name,
                transfer,
                override_value: i16_at(data, curve_off + 26),
            }
        })
        .collect();
    let dot_gain = u16_at(data, 412);
    let overprints = (0..OVERPRINTS_PER_PLATES[plates_usize - 1])
        .map(|j| color_at(data, 414 + j * 10))
        .collect();
    Some(DuotoneSpec {
        version,
        plates,
        inks,
        dot_gain,
        overprints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u16(data: &mut [u8], off: usize, v: u16) {
        data[off..off + 2].copy_from_slice(&v.to_be_bytes());
    }

    fn put_name(data: &mut [u8], off: usize, name: &str) {
        let bytes = name.as_bytes();
        data[off] = bytes.len() as u8;
        data[off + 1..off + 1 + bytes.len()].copy_from_slice(bytes);
    }

    fn two_plate_buffer() -> Vec<u8> {
        let mut d = vec![0u8; DUOTONE_DATA_SIZE];
        put_u16(&mut d, 0, 1); // version
        put_u16(&mut d, 2, 2); // plates
        put_u16(&mut d, 4, 1); // ink 0 space: RGB
        put_u16(&mut d, 6, 255);
        put_u16(&mut d, 8, 0);
        put_u16(&mut d, 10, 0);
        put_u16(&mut d, 12, 255);
        put_u16(&mut d, 14, 4); // ink 1 space: CMYK
        put_u16(&mut d, 16, 0);
        put_u16(&mut d, 18, 255);
        put_u16(&mut d, 20, 255);
        put_u16(&mut d, 22, 0);
        put_name(&mut d, 44, "PANTONE 185");
        put_name(&mut d, 44 + 64, "Black");
        // ink 0 curve: 0, -1, 500, ..., 1000
        put_u16(&mut d, 300, 0);
        put_u16(&mut d, 302, (-1i16) as u16);
        put_u16(&mut d, 304, 500);
        put_u16(&mut d, 300 + 24, 1000);
        put_u16(&mut d, 300 + 26, 7); // override
        put_u16(&mut d, 412, 20); // dot gain
        put_u16(&mut d, 414, 1); // one overprint, space 1
        put_u16(&mut d, 416, 10);
        put_u16(&mut d, 418, 20);
        put_u16(&mut d, 420, 30);
        put_u16(&mut d, 422, 40);
        d
    }

    #[test]
    fn two_plate_spec_parses() {
        let spec = parse_duotone(&two_plate_buffer()).unwrap();
        assert_eq!(spec.version, 1);
        assert_eq!(spec.plates, 2);
        assert_eq!(spec.inks.len(), 2);
        assert_eq!(spec.inks[0].name, "PANTONE 185");
        assert_eq!(spec.inks[1].name, "Black");
        assert_eq!(spec.inks[0].color.space, 1);
        assert_eq!(spec.inks[0].color.components, [255, 0, 0, 255]);
        assert_eq!(spec.inks[0].transfer[0], 0);
        assert_eq!(spec.inks[0].transfer[1], -1);
        assert_eq!(spec.inks[0].transfer[2], 500);
        assert_eq!(spec.inks[0].transfer[12], 1000);
        assert_eq!(spec.inks[0].override_value, 7);
        assert_eq!(spec.dot_gain, 20);
        assert_eq!(spec.overprints.len(), 1);
        assert_eq!(spec.overprints[0].components, [10, 20, 30, 40]);
    }

    #[test]
    fn short_or_invalid_blocks_are_rejected() {
        assert!(parse_duotone(&[0u8; 10]).is_none());
        let mut short = two_plate_buffer();
        short.truncate(DUOTONE_DATA_SIZE - 1);
        assert!(parse_duotone(&short).is_none());
        for plates in [0u16, 5, 65535] {
            let mut d = two_plate_buffer();
            put_u16(&mut d, 2, plates);
            assert!(parse_duotone(&d).is_none(), "plates {plates}");
        }
    }
}
