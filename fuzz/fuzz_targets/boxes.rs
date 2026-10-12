//! The box walk every other parser stands on -- where the 0.1.1 `uuid` panic was. Each box's
//! payload is walked again as a container, as `meta` and `iprp` are.
#![no_main]

use heic_rs::boxes::BoxIter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    for item in BoxIter::new(data) {
        let Ok(header) = item else { break };
        for inner in BoxIter::new(header.payload) {
            if inner.is_err() {
                break;
            }
        }
    }
});
