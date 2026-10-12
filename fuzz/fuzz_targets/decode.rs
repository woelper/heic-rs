//! The whole decoder: any input is an image or an `Error`, never a panic or a hang. `max_pixels` is
//! held to four megapixels, so a legitimately large image is refused rather than mistaken for a
//! resource-exhaustion finding; everything else is the default a caller decoding uploads uses.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let options = heic_rs::DecodeOptions {
        max_pixels: Some(1 << 22),
        ..heic_rs::DecodeOptions::default()
    };
    let _ = heic_rs::decode(data, &options);
});
