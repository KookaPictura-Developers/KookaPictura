//! Verification harness for Kooka Pictura.
//!
//! M0 scope: a deterministic golden-image comparison. The comparison model is
//! specified in `docs/11-cross-cutting/testing-strategy.md`: decode both sides
//! into one comparison space (planar, 8-bit is enough for M0) and apply a
//! per-operation-class tolerance.

/// Result of comparing two same-shaped buffers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// Total samples compared.
    pub samples: usize,
    /// Samples that differ by more than the tolerance.
    pub differing: usize,
    /// Largest absolute difference observed.
    pub max_delta: u16,
    /// Sum of absolute differences (for a mean metric).
    pub sum_delta: u64,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.differing == 0
    }

    pub fn mean_delta(&self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.sum_delta as f64 / self.samples as f64
        }
    }
}

/// Compare two byte buffers of equal length with an absolute per-sample
/// tolerance. Returns `Err` if lengths differ.
pub fn compare(a: &[u8], b: &[u8], tolerance: u8) -> Result<Diff, String> {
    if a.len() != b.len() {
        return Err(format!(
            "length mismatch: {} vs {}",
            a.len(),
            b.len()
        ));
    }
    let mut differing = 0usize;
    let mut max_delta = 0u16;
    let mut sum_delta = 0u64;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = x.abs_diff(*y);
        if d > tolerance {
            differing += 1;
        }
        max_delta = max_delta.max(d as u16);
        sum_delta += d as u64;
    }
    Ok(Diff {
        samples: a.len(),
        differing,
        max_delta,
        sum_delta,
    })
}

/// Stable content hash for determinism checks.
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    // FNV-1a: tiny, dependency-free, good enough for M0 determinism checks.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
