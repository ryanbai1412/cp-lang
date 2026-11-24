//! The checker's database, kept per thread, and when a run gives it back.

use std::sync::atomic::{AtomicBool, Ordering};

/// The thread's database. One per thread, as the language server keeps one:
/// a process that compiles many files (the test suites, `cpl watch`) checks
/// the standard library once, and a command that asks more of a file it
/// compiled (`explain`'s storage) doesn't check it again. The loader rereads
/// imports on every analysis.
pub(crate) fn with_db<R>(f: impl FnOnce(&mut cpl_check::Session) -> R) -> R {
    DB.with_borrow_mut(f)
}

/// The thread's session as lowering's source of body facts.
pub(crate) struct DbFacts;

impl cpl_check::FactsSource for DbFacts {
    fn body_facts(
        &self,
        path: &str,
        at: cpl_diag::Span,
    ) -> Option<std::sync::Arc<cpl_check::Facts>> {
        with_db(|db| db.body_facts(path, at))
    }
}

thread_local! {
    static DB: std::cell::RefCell<cpl_check::Session> = Default::default();
}

static ONE_COMMAND: AtomicBool = AtomicBool::new(false);

/// Tells the library its host runs one command and exits, as the `cpl`
/// binary does: nothing will compile again, so a run gives the checked
/// standard library back before the program starts, the way a C++ build's
/// compiler has exited before its binary runs.
pub fn one_command_per_process() {
    ONE_COMMAND.store(true, Ordering::Relaxed);
}

/// Drops the thread's database, in a one-command process, so the program
/// runs without the front end's memory beside it.
pub(crate) fn release() {
    if ONE_COMMAND.load(Ordering::Relaxed) {
        drop(DB.take());
    }
}
