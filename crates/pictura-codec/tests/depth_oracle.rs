//! Differential tests for 16- and 32-bit composites narrowed to the 8-bit
//! working model. The committed fixtures (authored by
//! `scripts/generate-fixtures.py`) are re-read with the independent `psd-tools`
//! decoder and compared exactly (0 tolerance). The oracle self-skips when
//! `psd-tools` is absent.

use std::path::PathBuf;
use std::process::Command;

use pictura_codec::{read_psd, write_psd};
use pictura_core::{BitDepth, ColorMode, Document};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load(name: &str) -> Document {
    let path = fixture_dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {} ({e}); run `python3 scripts/generate-fixtures.py`",
            path.display()
        )
    });
    read_psd(&bytes).unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()))
}

fn psd_tools_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("python3")
            .args(["-c", "import psd_tools"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// Decode `path` with `psd-tools` and return its composite as planar RGB hex.
fn psd_tools_composite_planes(path: &std::path::Path) -> Vec<String> {
    let script = r#"
import sys
import numpy as np
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
arr = np.array(psd.composite().convert("RGB"))
for c in range(3):
    print(arr[:, :, c].tobytes().hex())
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(path)
        .output()
        .expect("run python3");
    assert!(
        out.status.success(),
        "psd-tools failed on {}:\n{}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Every narrowed 8-bit pixel is byte-identical to psd-tools' composite.
fn assert_matches_psd_tools(name: &str) {
    let path = fixture_dir().join(name);
    let doc = load(name);
    let plane = doc.composite.pixel_count();
    let theirs = psd_tools_composite_planes(&path);
    assert_eq!(theirs.len(), 3, "psd-tools printed three RGB planes");
    for (c, expected) in theirs.iter().enumerate() {
        assert_eq!(
            hex(&doc.composite.data[c * plane..(c + 1) * plane]),
            *expected,
            "{name}: plane {c}"
        );
    }
}

#[test]
fn depth16_fixture_matches_psd_tools_exactly() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    assert_matches_psd_tools("rgb16.psd");
}

#[test]
fn depth32_fixture_matches_psd_tools_exactly() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    assert_matches_psd_tools("rgb32.psd");
}

/// A normalized depth read saves as 8-bit: re-reading the written file yields
/// the same pixels with no source depth, and `psd-tools` opens it as 8-bit RGB.
#[test]
fn normalized_depth_documents_round_trip_as_8bit() {
    for (name, source) in [
        ("rgb16.psd", BitDepth::Sixteen),
        ("rgb32.psd", BitDepth::ThirtyTwo),
    ] {
        let doc = load(name);
        assert_eq!(doc.mode, ColorMode::Rgb, "{name}: working mode");
        assert_eq!(doc.depth, BitDepth::Eight, "{name}: normalized depth");
        assert_eq!(
            doc.source_depth,
            Some(source),
            "{name}: recorded source depth"
        );

        let out = write_psd(&doc).unwrap();
        assert_eq!(
            u16::from_be_bytes(out[22..24].try_into().unwrap()),
            8,
            "{name}: output header depth is 8"
        );
        let back = read_psd(&out).unwrap();
        assert_eq!(
            back.source_depth, None,
            "{name}: no source depth after save"
        );
        assert_eq!(
            back.composite, doc.composite,
            "{name}: normalized pixels are stable across a save"
        );

        if !psd_tools_available() {
            eprintln!("skipping: python3 + psd-tools not available");
            continue;
        }
        let dir = scratch_dir("depth-roundtrip");
        let path = dir.join(name);
        std::fs::write(&path, &out).unwrap();
        let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
print(int(psd.depth), int(psd.color_mode))
"#;
        let result = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&path)
            .output()
            .expect("run python3");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            result.status.success(),
            "psd-tools failed on the written {name}:\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim(),
            "8 3",
            "{name}: psd-tools opens the output as 8-bit RGB"
        );
    }
}

/// A 1x1 RGB document at `depth` with one layer carrying a single unmodeled
/// `id` channel (a spot/selection channel) whose raw samples are `plane`. The
/// layer has no color channels, so `psd-tools` sees exactly this channel.
fn layered_spot_psd(depth: u16, id: i16, plane: &[u8]) -> Vec<u8> {
    let mut rec = Vec::new();
    for v in [0i32, 0, 1, 1] {
        rec.extend_from_slice(&v.to_be_bytes());
    }
    rec.extend_from_slice(&1u16.to_be_bytes()); // one layer channel
    rec.extend_from_slice(&id.to_be_bytes());
    rec.extend_from_slice(&((2 + plane.len()) as u32).to_be_bytes());
    rec.extend_from_slice(b"8BIMnorm");
    rec.extend_from_slice(&[255, 0, 0, 0]); // opacity, clipping, flags, filler
    let extra = {
        let mut e = Vec::new();
        e.extend_from_slice(&0u32.to_be_bytes()); // no mask
        e.extend_from_slice(&0u32.to_be_bytes()); // blending ranges
        e.extend_from_slice(&[1, b'L', 0, 0]); // pascal name "L"
        e
    };
    rec.extend_from_slice(&(extra.len() as u32).to_be_bytes());
    rec.extend_from_slice(&extra);

    let mut info = Vec::new();
    info.extend_from_slice(&1i16.to_be_bytes());
    info.extend_from_slice(&rec);
    info.extend_from_slice(&0u16.to_be_bytes()); // raw channel compression
    info.extend_from_slice(plane);
    while info.len() % 4 != 0 {
        info.push(0);
    }

    let mut out = Vec::new();
    out.extend_from_slice(b"8BPS");
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&[0u8; 6]);
    out.extend_from_slice(&3u16.to_be_bytes()); // RGB
    out.extend_from_slice(&1u32.to_be_bytes()); // height
    out.extend_from_slice(&1u32.to_be_bytes()); // width
    out.extend_from_slice(&depth.to_be_bytes());
    out.extend_from_slice(&3u16.to_be_bytes()); // color mode code
    out.extend_from_slice(&0u32.to_be_bytes()); // color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // image resources
    let section_len = 4 + info.len() + 4;
    out.extend_from_slice(&(section_len as u32).to_be_bytes());
    out.extend_from_slice(&(info.len() as u32).to_be_bytes());
    out.extend_from_slice(&info);
    out.extend_from_slice(&0u32.to_be_bytes()); // global layer mask
    out.extend_from_slice(&0u16.to_be_bytes()); // raw composite compression
                                                // Three RGB planes at the document's sample width (the pixel values are
                                                // irrelevant to this test; only the layer channel is checked).
    out.extend(std::iter::repeat_n(0u8, 3 * (depth as usize / 8)));
    out
}

/// The HIGH fix: a normalized document's unmodeled layer channel is narrowed,
/// so the 8-bit save carries sane bytes (not the source-depth payload) and
/// `psd-tools` decodes it as a 1-byte 8-bit raw channel.
#[test]
fn normalized_unmodeled_layer_channel_saves_as_8bit() {
    for (depth, plane, sample) in [
        (16u16, 0xab00u16.to_be_bytes().to_vec(), 0xabu8),
        (32u16, 0.5f32.to_be_bytes().to_vec(), 128u8),
        (16u16, 0x0100u16.to_be_bytes().to_vec(), 0x01u8),
    ] {
        let input = layered_spot_psd(depth, 3, &plane);
        let doc = read_psd(&input).unwrap();
        assert_eq!(
            doc.layers[0].raw_channels[0].data,
            vec![0, 0, sample],
            "depth {depth}: channel decoded and narrowed to an 8-bit raw stream"
        );

        let out = write_psd(&doc).unwrap();
        assert_eq!(
            u16::from_be_bytes(out[22..24].try_into().unwrap()),
            8,
            "depth {depth}: output header is 8-bit"
        );
        let back = read_psd(&out).unwrap();
        assert_eq!(
            back.layers[0].raw_channels[0].data,
            vec![0, 0, sample],
            "depth {depth}: the 8-bit stream re-reads unchanged"
        );

        if !psd_tools_available() {
            eprintln!("skipping: python3 + psd-tools not available");
            continue;
        }
        let dir = scratch_dir("unmodeled-depth");
        let path = dir.join(format!("spot{depth}-{sample}.psd"));
        std::fs::write(&path, &out).unwrap();
        let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
layer = psd[0]
ch = layer._channels[0]
info = layer._record.channel_info[0]
print(int(psd.depth), int(psd.color_mode), int(info.id), int(ch.compression), ch.data.hex())
"#;
        let result = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&path)
            .output()
            .expect("run python3");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            result.status.success(),
            "psd-tools failed on the written depth-{depth} file:\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim(),
            format!("8 3 3 0 {:02x}", sample),
            "depth {depth}: psd-tools reads 8-bit RGB with a raw 8-bit spot channel"
        );
    }
}

fn scratch_dir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pictura-codec-depth-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
