//! Independent ag-psd structural oracle for `mixr` (Channel Mixer) payloads.
//!
//! psd-tools' typed `ChannelMixer` reads only the red row (the rest lands in
//! its `unknown` blob), so it cannot see the green/blue/gray channels. ag-psd
//! reads all four channels, so this test runs it over the committed
//! `channel_mixer.psd` and asserts the authored values.
//!
//! Needs `node` with the `ag-psd` npm package resolvable from the crate's
//! parents: `npm i ag-psd` at the repo root, or point `NODE_PATH` at an existing
//! install. The test self-skips with a message when either is absent, mirroring
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
