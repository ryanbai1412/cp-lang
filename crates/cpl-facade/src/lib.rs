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

#[cfg(feature = "cpp")]
mod cpp;
mod editor;
mod library;
mod run;
mod session;
mod snapshot;
pub mod today;

pub use cpl_diag::{Diagnostic, Severity};
#[cfg(feature = "cpp")]
pub use cpp::{
    emit_cpp, emit_cpp_profiled, grader_header, Choice, CppFile, Part, Profile, Size, SizeBar,
};
pub use editor::{Definition, Hover, Typed};
#[cfg(feature = "cpp")]
pub use library::COMPONENTS;
pub use library::{file_path, libraries, Std};
pub use run::{
    run, run_checkpointed, run_for, run_to_end, run_traced, run_traced_within, run_within,
    Checkpoint, Checkpointed, Checkpoints, Choices, Limits, More, Outcome, Recording, Request,
    RunInput, RunProfile, Stop, TraceConfig, Trap, CANCELLED, OUT_OF_TIME,
};
pub use session::{
    apply_error_fixes, has_judge, Branch, Checks, Job, Layer, Program, Roles, Root, Session,
    JUDGE_DECORATORS,
};
pub use snapshot::{FileId, Snapshot, Text, World};
