//! The one place cpl-cli stops on a fact its own code guarantees.

/// Stops because `held`, a fact the code guarantees, did not hold.
#[cold]
#[track_caller]
pub fn broken_invariant(held: &'static str) -> ! {
    panic!("broken invariant: {held}")
}
