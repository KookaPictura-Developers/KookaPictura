use super::*;

pub const FIXTURES: &[(&str, u32, u32, ColorMode)] = &[
    ("two_layers.psd", 8, 8, ColorMode::Rgb),
    ("group.psd", 8, 8, ColorMode::Rgb),
    ("masked.psd", 8, 8, ColorMode::Rgb),
    ("gray.psd", 8, 8, ColorMode::Grayscale),
    ("adjustment.psd", 8, 8, ColorMode::Rgb),
    ("gradient_map.psd", 8, 8, ColorMode::Rgb),
    ("solid_fill.psd", 8, 8, ColorMode::Rgb),
    ("gradient_fill.psd", 8, 8, ColorMode::Rgb),
    ("pattern_fill.psd", 8, 8, ColorMode::Rgb),
    ("pattern_fill_16bit.psd", 8, 8, ColorMode::Rgb),
    ("drop_shadow.psd", 8, 8, ColorMode::Rgb),
    ("outer_glow.psd", 8, 8, ColorMode::Rgb),
    ("inner_shadow.psd", 8, 8, ColorMode::Rgb),
    ("inner_glow.psd", 8, 8, ColorMode::Rgb),
    ("stroke.psd", 8, 8, ColorMode::Rgb),
    ("stroke_gradient.psd", 8, 8, ColorMode::Rgb),
    ("stroke_pattern.psd", 8, 8, ColorMode::Rgb),
    ("color_overlay.psd", 8, 8, ColorMode::Rgb),
    ("gradient_overlay.psd", 8, 8, ColorMode::Rgb),
    ("pattern_overlay.psd", 8, 8, ColorMode::Rgb),
    ("satin.psd", 8, 8, ColorMode::Rgb),
    ("bevel.psd", 8, 8, ColorMode::Rgb),
    ("legacy_effects.psd", 8, 8, ColorMode::Rgb),
];

pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn load(name: &str) -> pictura_core::Document {
    let path = fixture_dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {} ({e}); run `python3 scripts/generate-fixtures.py`",
            path.display()
        )
    });
    read_psd(&bytes).unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()))
}

pub fn psd_tools_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("python3")
            .args(["-c", "import psd_tools"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// Unique scratch directory per call so tests can run in parallel.
pub fn scratch_dir(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pictura-codec-oracle-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
