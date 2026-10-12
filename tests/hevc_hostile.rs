//! Malformed HEVC streams are refused, never a panic -- in release builds and with overflow checks
//! on (debug builds, tests, fuzzing).
//!
//! Five files from the fuzzing run of s17sh70x2jfl031ag, reported privately in October 2026
//! against 0.1.3, each with a tested fix. The first three panicked in release builds; the last two
//! overflowed with overflow checks on and returned an error without them.

/// Decodes `bytes` with the defaults a caller decoding uploads would use, and says whether it
/// panicked rather than answering.
fn panics(bytes: &[u8]) -> bool {
    std::panic::catch_unwind(|| heic_rs::decode(bytes, &heic_rs::DecodeOptions::default())).is_err()
}

/// A block starting right of the picture: its width clipped to nothing, its row offsets past the
/// end of the plane (`decode/recon.rs`).
#[test]
fn a_block_right_of_the_picture_writes_nothing() {
    assert!(!panics(include_bytes!("fixtures/fuzz/heic-rs-bug4.heic")));
}

/// A conformance window as large as the picture: the crop started outside the decoded plane
/// (`decode/mod.rs`, from `ps/sps_parse.rs`).
#[test]
fn a_conformance_window_larger_than_the_picture_is_refused() {
    let file = include_bytes!("fixtures/fuzz/heic-rs-bug5.heic");
    assert!(!panics(file));
    assert!(heic_rs::decode(file, &heic_rs::DecodeOptions::default()).is_err());
}

/// A chroma block starting below the plane: its top reference row lay past the end of it
/// (`decode/refs.rs`).
#[test]
fn a_reference_row_below_the_picture_is_unavailable() {
    assert!(!panics(include_bytes!("fixtures/fuzz/heic-rs-bug6.heic")));
}

/// A PCM bit depth above the sample bit depth: `bit_depth - pcm_bit_depth` underflowed
/// (`decode/ctu.rs`, from `ps/sps_parse.rs`).
#[test]
fn a_pcm_bit_depth_above_the_sample_bit_depth_is_refused() {
    let file = include_bytes!("fixtures/fuzz/heic-rs-bug2.heic");
    assert!(!panics(file));
    assert!(heic_rs::decode(file, &heic_rs::DecodeOptions::default()).is_err());
}

/// A slice QP outside `-QpBdOffsetY..=51`: the dequantiser shifted by 64 or more
/// (`transform/dequant.rs`, from `slice.rs`).
#[test]
fn a_slice_qp_out_of_range_is_refused() {
    let file = include_bytes!("fixtures/fuzz/heic-rs-bug3.heic");
    assert!(!panics(file));
    assert!(heic_rs::decode(file, &heic_rs::DecodeOptions::default()).is_err());
}
