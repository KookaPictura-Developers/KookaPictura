use super::*;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

fn zlib(data: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::fast());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

#[test]
fn zip_bomb_is_rejected_with_bounded_allocation() {
    // 256 MiB of zeros compresses to a few hundred KiB. The bounded decode
    // reads at most `expected + 1` bytes, so the expansion is never
    // materialised; before the guard this allocated the whole 256 MiB (and
    // a real bomb scales without limit).
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::fast());
    let chunk = vec![0u8; 1 << 20];
    for _ in 0..256 {
        enc.write_all(&chunk).unwrap();
    }
    let bomb = enc.finish().unwrap();
    assert!(
        bomb.len() < 4 << 20,
        "bomb input stays small: {}",
        bomb.len()
    );

    let err = inflate(&bomb, 4).unwrap_err();
    assert!(matches!(err, PsdError::Invalid(_)), "typed error: {err:?}");
}

#[test]
fn zip_exact_is_ok_and_both_mismatches_error() {
    assert_eq!(inflate(&zlib(&[1, 2, 3, 4]), 4).unwrap(), vec![1, 2, 3, 4]);
    assert!(matches!(
        inflate(&zlib(&[1, 2]), 4).unwrap_err(),
        PsdError::Invalid(_)
    ));
    // Over-long output is malformed, not silently truncated.
    assert!(matches!(
        inflate(&zlib(&[1, 2, 3, 4, 5]), 4).unwrap_err(),
        PsdError::Invalid(_)
    ));
    // A bare deflate stream (no zlib framing) still decodes via the fallback.
    use flate2::write::DeflateEncoder;
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::fast());
    enc.write_all(&[9, 9, 9]).unwrap();
    assert_eq!(inflate(&enc.finish().unwrap(), 3).unwrap(), vec![9, 9, 9]);
}
