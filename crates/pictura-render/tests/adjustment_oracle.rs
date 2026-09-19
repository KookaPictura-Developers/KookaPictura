//! psd-tools structural oracle for `pictura_render::encode_color_balance`.
//!
//! Unlike the fixture-based adjustment oracles, this does not touch a golden
//! file: it encodes a `blnc` block in Rust and lets psd-tools' independent
//! `ColorBalance` reader decode the same bytes, asserting the four fields match
//! the encoder inputs. Self-skips when `python3`/`psd_tools` is absent.

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
