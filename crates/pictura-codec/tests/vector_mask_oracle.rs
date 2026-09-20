//! Independent `psd-tools` oracle for the `vmsk` vector-mask fixture.
//!
//! `oracle.rs` is at its 1400-line cap, so this check lives in its own binary.
//! It reads `vector_mask.psd` with `psd-tools` (not with `pictura-codec`) and
//! reports each vector mask's version, flags, closed subpath, and the
//! even-odd pixel coverage of the authored rectangle.
//!
//! Needs `python3` with `psd-tools`; self-skips with a message otherwise.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const SCRIPT: &str = r#"
import sys
from psd_tools import PSDImage
from psd_tools.constants import Tag

psd = PSDImage.open(sys.argv[1], lazy=False)
w, h = psd.width, psd.height
lines = []
for layer in psd:
    blocks = layer.tagged_blocks
    block = blocks.get(Tag.VECTOR_MASK_SETTING1) if blocks else None
    if block is None:
        continue
    mask = block.data
    closed = [item for item in mask.path if type(item).__name__ == "ClosedPath"]
    knots = []
    for subpath in closed:
        for knot in subpath:
            knots.append((knot.anchor[1] * w, knot.anchor[0] * h))
    inside = 0
    for py in range(h):
        for px in range(w):
            cx, cy = px + 0.5, py + 0.5
            crossings = 0
            for i in range(len(knots)):
                x0, y0 = knots[i]
                x1, y1 = knots[(i + 1) % len(knots)]
                if (y0 > cy) != (y1 > cy):
                    xint = x0 + (cy - y0) * (x1 - x0) / (y1 - y0)
                    if cx < xint:
                        crossings += 1
            if crossings % 2 == 1:
                inside += 1
    operation = closed[0].operation if closed else -99
    fields = [layer.name, str(mask.version), str(mask.flags), str(len(closed)),
              str(operation), str(inside)]
    fields.append(" ".join(f"{x:g},{y:g}" for x, y in knots))
    lines.append("|".join(fields))
print("\n".join(lines))
"#;

fn psd_tools_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("python3")
            .args(["-c", "import psd_tools"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

#[test]
fn psd_tools_reads_vector_mask_fixture() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vector_mask.psd");
    let out = Command::new("python3")
        .args(["-c", SCRIPT, fixture.to_str().expect("utf-8 fixture path")])
        .output()
        .expect("run python3");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "psd-tools failed:\n{stdout}\n{stderr}"
    );

    let lines: Vec<Vec<&str>> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split('|').collect())
        .collect();
    assert_eq!(lines.len(), 2, "two vector-mask layers:\n{stdout}");

    assert_eq!(
        lines[0],
        vec![
            "Shape Inverted",
            "3",
            "1",
            "1",
            "1",
            "36",
            "0,0 6,0 6,6 0,6"
        ],
        "inverted (0,0)-(6,6) rectangle: {stdout}"
    );
    assert_eq!(
        lines[1],
        vec!["Shape", "3", "0", "1", "1", "4", "1,1 3,1 3,3 1,3"],
        "normal (1,1)-(3,3) rectangle: {stdout}"
    );
}
