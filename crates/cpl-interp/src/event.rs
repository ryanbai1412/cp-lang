//! The one model of what a run does, as the VM and the tree-walker emit it:
//! every observer (the tracer, the profiler, the VM's fallback counts, the
//! debugger and replay through the trace) hears these events through
//! [`Observer::on`](crate::observe::Observer::on) and nothing else.

use crate::trace::{IntWrite, View};
use cpl_diag::Span;
use cpl_ir::{Place, Stmt};

/// One event of a run. A `view` is the run's state when it happened; an
/// `*Unseen` write comes with no view, for an observer that can take note
/// of it without one ([`Observer::on`](crate::observe::Observer::on)
/// returns true when it did).
pub(crate) enum Event<'e, 'v> {
    /// A user statement (one of [`cpl_ir::sites::user_stmts`]) starts: the
    /// step a profiler charges time between.
    Step(&'e Stmt),
    /// The tree-walker ran `stmt`: what deleting it still has to move.
    Walked(&'e Stmt),
    /// The VM left what `tag` names to the tree-walker, in a loop when `in_loop`.
    /// Native only: `CPL_VM_STATS` is its one reader.
    #[cfg(not(target_family = "wasm"))]
    FellBack {
        tag: &'e dyn Fn() -> String,
        in_loop: bool,
    },
    /// An internal call starts (`true`) or ends; the observer answers true
    /// when it stops hearing the user's steps itself, false to be taken
    /// away for the call.
    Hush(bool),
    /// The run is a multitest; its cases come next.
    BeginCases,
    /// The output between two cases.
    Divider,
    StartCase(u64),
    /// A run from a checkpoint starts inside test case `case`.
    ResumeCase(u64),
    /// A fn is entered, until the matching [`Event::LeaveFn`].
    EnterFn(&'e str),
    LeaveFn,
    /// The running `yields` fn feeds its loop's body, until [`Event::LeaveFedBody`].
    EnterFedBody,
    LeaveFedBody,
    /// A call the trace stepped over, from `at`, took a `mut` argument.
    SteppedOver {
        view: &'e View<'v>,
        at: Span,
    },
    /// A fn value ran: any global may have changed, through a capture or a
    /// `mut` argument.
    GlobalsMayChange,
    EnterCall {
        name: &'e str,
        args: &'e dyn Fn() -> String,
    },
    LeaveCall(Option<String>),
    /// Global `id` was given its first value.
    Declared {
        id: usize,
        view: &'e View<'v>,
        at: Span,
    },
    /// A cell of global `id`, at `path`, changed.
    Changed {
        id: usize,
        view: &'e View<'v>,
        at: Span,
        path: &'e [usize],
    },
    /// Global `id` was mutated in place, by a method call.
    Mutated {
        id: usize,
        view: &'e View<'v>,
        at: Span,
    },
    /// Local `name` of the running call changed.
    LocalChanged {
        view: &'e View<'v>,
        at: Span,
        name: &'e str,
    },
    AssignedLocal {
        view: &'e View<'v>,
        at: Span,
        name: &'e str,
    },
    AssignedGlobal {
        view: &'e View<'v>,
        at: Span,
        id: usize,
    },
    Printed {
        view: &'e View<'v>,
        at: Span,
        text: &'e str,
    },
    Debugged {
        view: &'e View<'v>,
        at: Span,
        name: &'e str,
        text: String,
        json: String,
    },
    /// Global `place`, or a part of it, was written where no step is kept.
    QuietWrite {
        place: &'e Place,
        depth: usize,
    },
    /// [`Event::LocalChanged`] at call depth `depth`, with no view.
    LocalChangedUnseen {
        depth: usize,
        name: &'e str,
    },
    /// [`Event::Changed`] with no view.
    ChangedUnseen {
        id: usize,
        name: &'e str,
        depth: usize,
        path: &'e [usize],
    },
    /// [`Event::Declared`] with no view.
    DeclaredUnseen {
        id: usize,
        name: &'e str,
        unnamed: bool,
        depth: usize,
        at: Span,
    },
    /// [`Event::Declared`] or [`Event::Changed`] of an `Int` global or cell,
    /// with no view; an observer that does not take it asks for the view.
    IntWrite(&'e IntWrite<'v>),
}
