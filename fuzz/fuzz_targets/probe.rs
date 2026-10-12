//! The container alone, as `probe` reads it before any pixel is decoded.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = heic_rs::probe(data);
});
