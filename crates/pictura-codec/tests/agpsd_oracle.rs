//! Independent ag-psd structural oracle for `mixr` (Channel Mixer), `curv`
//! (Curves), and `selc` (Selective Color) payloads.
//!
//! psd-tools' typed `ChannelMixer` reads only the red row (the rest lands in
//! its `unknown` blob), so it cannot see the green/blue/gray channels; its
//! `Curves` reads the v1 prefix but does not name the per-channel rows; its
//! `SelectiveColor` names no plate. ag-psd reads all three fully, so these
//! tests run it over the committed `channel_mixer.psd`, `curves.psd`, and
//! `selective_color.psd` fixtures and assert the authored values.
//!
//! Needs `node` with the `ag-psd` npm package resolvable from the crate's
//! parents: `npm i ag-psd` at the repo root, or point `NODE_PATH` at an existing
//! install. The tests self-skip with a message when either is absent, mirroring
//! the psd-tools/magick oracles.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const SCRIPT: &str = r#"
const fs = require('fs');
const ag = require('ag-psd');
const psd = ag.readPsd(fs.readFileSync(process.argv[1]), {
  skipLayerImageData: true,
  skipCompositeImageData: true,
});
const lines = [];
(function walk(layers) {
  for (const layer of layers || []) {
    const a = layer.adjustment;
    if (a && a.type === 'channel mixer') {
      if (a.monochrome) {
        const g = a.gray;
        lines.push([1, g.red, g.green, g.blue, g.constant].join(' '));
      } else {
        const r = a.red, g = a.green, b = a.blue, y = a.gray;
        lines.push([0, r.red, r.green, r.blue, r.constant, g.red, g.green, g.blue, g.constant,
          b.red, b.green, b.blue, b.constant, y.red, y.green, y.blue, y.constant].join(' '));
      }
    }
    walk(layer.children);
  }
})(psd.children);
console.log(lines.join('\n'));
"#;

fn node_ag_psd_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("node")
            .args(["-e", "require('ag-psd')"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

#[test]
fn ag_psd_reads_channel_mixer_fixture() {
    if !node_ag_psd_available() {
        eprintln!(
            "skipping ag-psd check: node + ag-psd not available (run `npm i ag-psd`, or set NODE_PATH)"
        );
        return;
    }

    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/channel_mixer.psd");
    let out = Command::new("node")
        .args(["-e", SCRIPT, fixture.to_str().expect("utf-8 fixture path")])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run node");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "ag-psd read failed:\n{stdout}\n{stderr}"
    );

    let lines: Vec<Vec<i64>> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(|token| token.parse::<i64>().expect("integer field"))
                .collect()
        })
        .collect();
    assert_eq!(lines.len(), 2, "two channel mixer layers:\n{stdout}");
    assert_eq!(
        lines[0],
        vec![0, 30, -10, 50, 5, 10, 90, 0, -20, 0, 20, 110, 40, 100, 0, 0, 0],
        "non-monochrome red/green/blue/gray: {stdout}"
    );
    assert_eq!(
        lines[1],
        vec![1, 20, 40, 60, -15],
        "monochrome gray: {stdout}"
    );
}

const CURVES_SCRIPT: &str = r#"
const fs = require('fs');
const ag = require('ag-psd');
const psd = ag.readPsd(fs.readFileSync(process.argv[1]), {
  skipLayerImageData: true,
  skipCompositeImageData: true,
});
const lines = [];
(function walk(layers) {
  for (const layer of layers || []) {
    const a = layer.adjustment;
    if (a && a.type === 'curves') {
      const parts = [];
      for (const name of ['rgb', 'red', 'green', 'blue']) {
        const c = a[name] || [];
        parts.push(c.length);
        for (const p of c) parts.push(p.output, p.input);
      }
      lines.push(parts.join(' '));
    }
    walk(layer.children);
  }
})(psd.children);
console.log(lines.join('\n'));
"#;

#[test]
fn ag_psd_reads_curves_fixture() {
    if !node_ag_psd_available() {
        eprintln!(
            "skipping ag-psd check: node + ag-psd not available (run `npm i ag-psd`, or set NODE_PATH)"
        );
        return;
    }

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/curves.psd");
    let out = Command::new("node")
        .args([
            "-e",
            CURVES_SCRIPT,
            fixture.to_str().expect("utf-8 fixture path"),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run node");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "ag-psd read failed:\n{stdout}\n{stderr}"
    );

    let lines: Vec<Vec<i64>> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(|token| token.parse::<i64>().expect("integer field"))
                .collect()
        })
        .collect();
    assert_eq!(lines.len(), 2, "two curves layers:\n{stdout}");
    assert_eq!(
        lines[0],
        vec![4, 0, 0, 32, 64, 224, 192, 255, 255, 0, 0, 0],
        "composite rgb (output, input) then empty red/green/blue: {stdout}"
    );
    assert_eq!(
        lines[1],
        vec![
            2, 0, 0, 255, 255, 3, 0, 0, 255, 128, 255, 255, 3, 0, 0, 16, 64, 255, 255, 2, 255, 0,
            0, 255
        ],
        "per-channel rgb/red/green/blue (output, input): {stdout}"
    );
}

const SELECTIVE_COLOR_SCRIPT: &str = r#"
const fs = require('fs');
const ag = require('ag-psd');
const psd = ag.readPsd(fs.readFileSync(process.argv[1]), {
  skipLayerImageData: true,
  skipCompositeImageData: true,
});
const ranges = ['reds', 'yellows', 'greens', 'cyans', 'blues', 'magentas', 'whites', 'neutrals', 'blacks'];
const lines = [];
(function walk(layers) {
  for (const layer of layers || []) {
    const a = layer.adjustment;
    if (a && a.type === 'selective color') {
      const parts = [a.mode === 'absolute' ? 1 : 0];
      for (const name of ranges) {
        const r = a[name];
        parts.push(r.c, r.m, r.y, r.k);
      }
      lines.push(parts.join(' '));
    }
    walk(layer.children);
  }
})(psd.children);
console.log(lines.join('\n'));
"#;

#[test]
fn ag_psd_reads_selective_color_fixture() {
    if !node_ag_psd_available() {
        eprintln!(
            "skipping ag-psd check: node + ag-psd not available (run `npm i ag-psd`, or set NODE_PATH)"
        );
        return;
    }

    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/selective_color.psd");
    let out = Command::new("node")
        .args([
            "-e",
            SELECTIVE_COLOR_SCRIPT,
            fixture.to_str().expect("utf-8 fixture path"),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run node");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "ag-psd read failed:\n{stdout}\n{stderr}"
    );

    let lines: Vec<Vec<i64>> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(|token| token.parse::<i64>().expect("integer field"))
                .collect()
        })
        .collect();
    assert_eq!(lines.len(), 2, "two selective color layers:\n{stdout}");
    assert_eq!(
        lines[0],
        vec![
            0, 10, -20, 30, 0, 0, 0, 0, 5, -10, 0, 0, 0, 0, 15, 0, 0, 0, 0, -25, 0, 5, 0, 0, 0, 0,
            0, 0, 0, 20, -10, 0, 0, 0, 0, 0, -40
        ],
        "relative reds..blacks: {stdout}"
    );
    assert_eq!(
        lines[1],
        vec![
            1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36
        ],
        "absolute reds..blacks: {stdout}"
    );
}
