//! An item built from other items cannot be made to outgrow the file that holds it.
//!
//! Found by the `probe` fuzz target on its first CI run (October 2026), against 0.1.2: 690 bytes
//! whose items are each assembled -- construction method 2, several extents apiece -- from the
//! items below them, eight deep. Every level multiplied the one beneath it, and `probe` asked for
//! 2.2 GB. The input is kept as `tests/fixtures/fuzz/probe-item-amplification.bin`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The system allocator, counting the most it has held at once.
struct Peak;

static NOW: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Peak {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let now = NOW.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
        PEAK.fetch_max(now, Ordering::SeqCst);
        // SAFETY: forwarded unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        NOW.fetch_sub(layout.size(), Ordering::SeqCst);
        // SAFETY: forwarded unchanged to the system allocator.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Peak = Peak;

#[test]
fn items_assembled_from_items_are_bounded_by_the_file() {
    let input = include_bytes!("fixtures/fuzz/probe-item-amplification.bin");
    let before = PEAK.load(Ordering::SeqCst);
    let probed = heic_rs::probe(input);
    let decoded = heic_rs::decode(input, &heic_rs::DecodeOptions::default());
    let grew = PEAK.load(Ordering::SeqCst).saturating_sub(before);
    assert!(probed.is_err(), "{probed:?}");
    assert!(decoded.is_err());
    // A 690-byte file has no business holding even a megabyte of item data.
    assert!(
        grew < 1 << 20,
        "peak allocation grew by {grew} bytes for a 690-byte input"
    );
}
