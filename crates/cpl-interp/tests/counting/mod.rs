//! The test binary's allocator, counting what it hands out, so a test
//! measures what a run really allocates rather than what it estimates.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;

static IN_USE: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        IN_USE.fetch_add(layout.size(), Ordering::Relaxed);
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        IN_USE.fetch_add(size, Ordering::Relaxed);
        IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// The bytes in use now, and how many allocations (and reallocations)
/// were made so far.
pub fn usage() -> (usize, usize) {
    (
        IN_USE.load(Ordering::Relaxed),
        CALLS.load(Ordering::Relaxed),
    )
}
