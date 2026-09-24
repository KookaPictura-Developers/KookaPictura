use super::*;

pub(super) fn exposure_payload(exposure: f32, offset: f32, gamma: f32) -> Vec<u8> {
    let mut data = 1u16.to_be_bytes().to_vec();
    data.extend_from_slice(&exposure.to_be_bytes());
    data.extend_from_slice(&offset.to_be_bytes());
    data.extend_from_slice(&gamma.to_be_bytes());
    data
}

pub(super) fn phfl_payload(
    version: u16,
    components: [u16; 4],
    density: u32,
    luminosity: u8,
) -> Vec<u8> {
    let mut data = version.to_be_bytes().to_vec();
    data.extend_from_slice(&0u16.to_be_bytes());
    for c in components {
        data.extend_from_slice(&c.to_be_bytes());
    }
    data.extend_from_slice(&density.to_be_bytes());
    data.push(luminosity);
    data.extend_from_slice(&[0, 0, 0]);
    data
}

pub(super) fn phfl_v3_payload(xyz: [u32; 3], density: u32, luminosity: u8) -> Vec<u8> {
    let mut data = 3u16.to_be_bytes().to_vec();
    for c in xyz {
        data.extend_from_slice(&c.to_be_bytes());
    }
    data.extend_from_slice(&density.to_be_bytes());
    data.push(luminosity);
    data.push(0);
    data
}

pub(super) fn blnc_payload(
    shadows: [i16; 3],
    midtones: [i16; 3],
    highlights: [i16; 3],
    luminosity: u8,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(20);
    for band in [shadows, midtones, highlights] {
        for value in band {
            data.extend_from_slice(&value.to_be_bytes());
        }
    }
    data.push(luminosity);
    data.push(0);
    data
}

pub(super) fn mixr_payload(monochrome: bool, channels: &[([i16; 3], i16)]) -> Vec<u8> {
    let mut data = 1u16.to_be_bytes().to_vec();
    data.extend_from_slice(&u16::from(monochrome).to_be_bytes());
    for (rgb, constant) in channels {
        for source in rgb {
            data.extend_from_slice(&source.to_be_bytes());
        }
        data.extend_from_slice(&[0, 0]);
        data.extend_from_slice(&constant.to_be_bytes());
    }
    data
}

pub(super) fn grdm_payload(
    version: u16,
    reverse: u8,
    dither: u8,
    name: &str,
    stops: &[(u32, [u16; 4])],
) -> Vec<u8> {
    let mut data = version.to_be_bytes().to_vec();
    data.push(reverse);
    data.push(dither);
    if version == 3 {
        data.extend_from_slice(b"Gcls");
    }
    let utf16: Vec<u16> = name.encode_utf16().collect();
    data.extend_from_slice(&(utf16.len() as u32).to_be_bytes());
    for u in utf16 {
        data.extend_from_slice(&u.to_be_bytes());
    }
    data.extend_from_slice(&(stops.len() as u16).to_be_bytes());
    for (location, color) in stops {
        data.extend_from_slice(&location.to_be_bytes());
        data.extend_from_slice(&50u32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        for c in color {
            data.extend_from_slice(&c.to_be_bytes());
        }
        data.extend_from_slice(&[0, 0]);
    }
    data
}

pub(super) fn desc_object(
    name: &str,
    class_id: &[u8],
    items: Vec<(Vec<u8>, DescValue)>,
) -> DescValue {
    DescValue::Object {
        name: name.to_string(),
        class_id: class_id.to_vec(),
        items,
    }
}
