//! The test binary's allocator, counting what each thread hands out, so a
//! test measures what a run really allocates rather than what it estimates.
//! The counts are per thread: a test measures a run it makes on its own
//! thread (`*_on_this_thread`), and a test running beside it cannot move
//! them.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// Hands each call to `System` unchanged, which
/// `the_counting_allocator_hands_each_call_to_the_system_one` checks.
struct Counting;

thread_local! {
    /// Bytes this thread allocated less those it freed. Signed: a thread
    /// may free what another allocated.
    static IN_USE: Cell<isize> = const { Cell::new(0) };
    static CALLS: Cell<usize> = const { Cell::new(0) };
}

/// Adds to this thread's counts; a thread being torn down has none left.
fn count(bytes: isize, calls: usize) {
    let _ = IN_USE.try_with(|n| n.set(n.get().wrapping_add(bytes)));
    let _ = CALLS.try_with(|n| n.set(n.get() + calls));
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size() as isize, 1);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        count(-(layout.size() as isize), 0);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count(size as isize - layout.size() as isize, 1);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// The bytes this thread has in use now, and how many allocations (and
/// reallocations) it made so far.
pub fn usage() -> (isize, usize) {
    (IN_USE.with(Cell::get), CALLS.with(Cell::get))
}

/// The stack a run on a test's own thread may fill.
pub const TEST_THREAD: cpl_interp::Limits = cpl_interp::Limits {
    stack_bytes: 1 << 20,
};

/// What `Counting` hands back holds what was written through every grow,
/// once freed the count is back where it was, and a thread allocating
/// beside it does not move it.
#[test]
fn the_counting_allocator_hands_each_call_to_the_system_one() {
    let both = std::sync::Arc::new(std::sync::Barrier::new(2));
    let beside = std::thread::spawn({
        let both = both.clone();
        move || {
            both.wait();
            let held: Vec<Vec<u8>> = (0..1000).map(|_| vec![0; 1000]).collect();
            both.wait();
            held.len()
        }
    });
    let (before, calls) = usage();
    both.wait();
    let mut v: Vec<u64> = (0..1000).collect();
    assert!(usage().0 >= before + 8000);
    v.extend(1000..100_000);
    assert!(v.iter().copied().eq(0..100_000));
    drop(v);
    both.wait();
    assert_eq!(usage().0, before);
    assert!(usage().1 > calls);
    assert_eq!(beside.join().ok(), Some(1000));
}
