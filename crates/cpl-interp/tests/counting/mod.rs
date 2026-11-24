//! The test binary's allocator, counting what each thread hands out, so a
//! test measures what a run really allocates rather than what it estimates.
//! The counts are per thread, so a test running beside it cannot move them,
//! and a run on another thread is not counted: `measure` is the only way to
//! read them, and its closure runs programs only through `OnThisThread`.

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
    /// Bytes this thread allocated, never less what it freed.
    static ALLOCATED: Cell<usize> = const { Cell::new(0) };
}

/// Adds to this thread's counts; a thread being torn down has none left.
fn count(bytes: isize, calls: usize) {
    let _ = IN_USE.try_with(|n| n.set(n.get().wrapping_add(bytes)));
    let _ = CALLS.try_with(|n| n.set(n.get() + calls));
    if bytes > 0 {
        let _ = ALLOCATED.try_with(|n| n.set(n.get() + bytes as usize));
    }
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

/// The bytes this thread has in use now, how many allocations (and
/// reallocations) it made so far, and the bytes they asked for.
fn usage() -> (isize, usize, usize) {
    (
        IN_USE.with(Cell::get),
        CALLS.with(Cell::get),
        ALLOCATED.with(Cell::get),
    )
}

/// What a measured closure allocated on this thread.
pub struct Usage {
    /// Bytes still in use when it returned.
    pub held: isize,
    /// Allocations and reallocations it made.
    pub calls: usize,
    /// Bytes those allocations asked for, freed or not.
    pub allocated: usize,
}

/// Runs programs on the measuring thread, the only thread `measure` counts.
pub struct OnThisThread {
    ran: Cell<bool>,
}

impl OnThisThread {
    pub fn observe(
        &self,
        program: &cpl_ir::Program,
        io: cpl_interp::Io,
        output: &mut (dyn std::io::Write + Send),
        trace: Option<cpl_interp::trace::TraceConfig>,
    ) -> cpl_interp::Observed {
        self.ran.set(true);
        let limits = cpl_interp::Limits {
            stack_bytes: 1 << 20,
        };
        cpl_interp::observe_on_this_thread(program, io, output, limits, trace)
    }
}

/// What `run` allocates on this thread. It fails unless `run` ran a program
/// through `OnThisThread`: a run on another thread would count nothing.
pub fn measure<T>(run: impl FnOnce(&OnThisThread) -> T) -> (T, Usage) {
    let here = OnThisThread {
        ran: Cell::new(false),
    };
    let (held, calls, allocated) = usage();
    let value = run(&here);
    let (now_held, now_calls, now_allocated) = usage();
    assert!(
        here.ran.get(),
        "measure: no program ran on this thread, and a run on another thread is not counted"
    );
    let usage = Usage {
        held: now_held - held,
        calls: now_calls - calls,
        allocated: now_allocated - allocated,
    };
    (value, usage)
}

/// A run measures what it allocated on this thread.
#[test]
fn measure_counts_a_run_on_this_thread() {
    let program = crate::common::program("let v = [i for i in 0..1000]\nprint(v[999])");
    let mut out = Vec::new();
    let (observed, usage) =
        measure(|here| here.observe(&program, cpl_interp::Io::fixed(b""), &mut out, None));
    assert!(observed.trap.is_none());
    assert_eq!(out, b"999\n");
    assert!(usage.calls > 0 && usage.allocated > 0 && usage.held < 1 << 20);
}

/// A closure whose work runs on another thread fails instead of counting 0.
#[test]
#[should_panic(expected = "no program ran on this thread")]
fn measure_fails_a_run_on_another_thread() {
    let _ = measure(|_| std::thread::scope(|s| s.spawn(|| vec![0u8; 1000].len()).join()));
}

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
    let (before, calls, _) = usage();
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
