//! psd-tools structural oracle for `pictura_render` adjustment encoders.
//!
//! Unlike the fixture-based adjustment oracles, these do not touch a golden
//! file: they encode a block in Rust and let psd-tools' independent reader
//! decode the same bytes, asserting the fields match. The `mixr` check reads
//! only the red row because psd-tools' `ChannelMixer` reads a `2H` + `5h`
//! prefix and dumps the green/blue/gray channels into its opaque `unknown`
//! blob; ag-psd (`crates/pictura-codec/tests/agpsd_oracle.rs`) is the
//! full-field oracle. Self-skips when `python3`/`psd_tools` is absent.

use std::process::Command;

use pictura_render::encode_color_balance;

/// Print `shadows midtones highlights luminosity` as whitespace-separated
/// integers, reading the hex-encoded `blnc` payload from `argv[1]`.
const SCRIPT: &str = "\
import sys
from io import BytesIO
from psd_tools.psd.adjustments import ColorBalance
cb = ColorBalance.read(BytesIO(bytes.fromhex(sys.argv[1])))
print(*cb.shadows, *cb.midtones, *cb.highlights, int(cb.luminosity))
";

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

#[test]
fn psd_tools_reads_encoded_color_balance() {
    if !psd_tools_available() {
        eprintln!("skipping psd-tools check: python3 + psd_tools not available");
        return;
    }

    let encoded = encode_color_balance(
        [-100.0, 0.0, 40.0],
        [25.0, 0.0, -30.0],
        [100.0, 0.0, 0.0],
        true,
    );
    assert_eq!(encoded.key, *b"blnc");
    assert_eq!(encoded.data.len(), 20);

    let hex: String = encoded.data.iter().map(|b| format!("{b:02x}")).collect();
    let out = Command::new("python3")
        .args(["-c", SCRIPT, &hex])
        .output()
        .expect("run python3");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "psd-tools ColorBalance read failed:\n{stdout}\n{stderr}"
    );

    let fields: Vec<i64> = stdout
        .split_whitespace()
        .map(|token| token.parse::<i64>().expect("integer field"))
        .collect();
    assert_eq!(
        fields,
        vec![-100, 0, 40, 25, 0, -30, 100, 0, 0, 1],
        "psd-tools read {fields:?}: {stdout}"
    );
}

/// Print `version monochrome` then the five shorts psd-tools reads from the
/// hex-encoded `mixr` payload in `argv[1]`.
const MIXR_SCRIPT: &str = "\
import sys
from io import BytesIO
from psd_tools.psd.adjustments import ChannelMixer
cm = ChannelMixer.read(BytesIO(bytes.fromhex(sys.argv[1])))
print(cm.version, cm.monochrome, *cm.data)
";

#[test]
fn psd_tools_reads_fixture_channel_mixer_prefix() {
    if !psd_tools_available() {
        eprintln!("skipping psd-tools check: python3 + psd_tools not available");
        return;
    }

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../pictura-codec/tests/fixtures/channel_mixer.psd"
    );
    let bytes = std::fs::read(path).expect("read channel_mixer.psd");
    let doc = pictura_codec::read_psd(&bytes).expect("fixture parses");
    let layer = doc
        .layers
        .iter()
        .find(|l| l.name == "Channel Mixer")
        .expect("Channel Mixer layer");
    let block = layer.adjustment.as_ref().expect("mixr block");
    assert_eq!(block.key, *b"mixr");
    assert_eq!(block.data.len(), 44);

    let hex: String = block.data.iter().map(|b| format!("{b:02x}")).collect();
    let out = Command::new("python3")
        .args(["-c", MIXR_SCRIPT, &hex])
        .output()
        .expect("run python3");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "psd-tools ChannelMixer read failed:\n{stdout}\n{stderr}"
    );

    // psd-tools sees only the red row plus the reserved pair and its constant:
    // version, monochrome, red.red, red.green, red.blue, reserved, red.constant.
    let fields: Vec<i64> = stdout
        .split_whitespace()
        .map(|token| token.parse::<i64>().expect("integer field"))
        .collect();
    assert_eq!(
        fields,
        vec![1, 0, 30, -10, 50, 0, 5],
        "psd-tools read {fields:?}: {stdout}"
    );
}
