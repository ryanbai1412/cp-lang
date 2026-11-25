//! The facade: the only interface a tool uses to check and run a program
//! (`handbook/design/next/contracts/09-facade.md`), over an immutable
//! [`Snapshot`] of the sources (`contracts/00-snapshot.md`).
//!
//! Today's compiler is behind it: `cpl-check`, `cpl-ir`, `cpl-interp` and
//! `cpl-codegen`. No tool crate depends on those (debt-ratchet's
//! `tool_compiler_deps`), so replacing them changes this crate and no tool.
//!
//! Every query is a function of the snapshot. Two tools that hold equal
//! snapshots get the same diagnostics, the same program and the same run:
//! there is one implementation of each, here.

mod run;
mod session;
mod snapshot;
pub mod today;

pub use cpl_diag::{Diagnostic, Severity};
pub use run::{
    run, run_for, run_to_end, Checkpoint, Checkpoints, run_traced, run_traced_within, run_within, Choices, Limits, More, Outcome, Recording, Request, RunInput, RunProfile, Stop,
    TraceConfig, Trap, CANCELLED, OUT_OF_TIME,
};
pub use session::{
    apply_error_fixes, emit_cpp, emit_cpp_profiled, Checks, CppFile, Job, Layer, Part, Profile, Program, Roles, Root, Session, Size,
    SizeBar,
};
pub use snapshot::{FileId, Snapshot, Text, World};
