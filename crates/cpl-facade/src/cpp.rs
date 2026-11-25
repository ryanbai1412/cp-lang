//! The C++ of a program, for a judge.

use crate::session::{Checks, Program, Session};

/// What the C++ is written for: the judge it is submitted to.
#[derive(Clone, Debug)]
pub struct Profile {
    /// A line for the header that names the judge.
    pub stamp: Option<String>,
    /// The stack the program runs with on a thread of its own; `None` on a
    /// judge that allows no threads.
    pub thread_stack: Option<u64>,
    /// Whether the judge runs Windows.
    pub windows: bool,
    pub checks: Checks,
    /// The judge's CPU features (`avx2`).
    pub cpu: Vec<String>,
    /// The source limit, when the judge has one.
    pub size: Option<SizeBar>,
    /// Whether the source opens the file as a comment. A file built to be
    /// run here, not submitted, leaves it out.
    pub source_comment: bool,
}

/// A judge's source limit: the file gives way (its source comment, then
/// its names) until it is at most `aim` bytes, and is refused over `limit`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizeBar {
    pub aim: usize,
    pub limit: usize,
}

impl Default for Profile {
    fn default() -> Profile {
        let target = cpl_codegen::Target::default();
        Profile {
            stamp: target.stamp,
            thread_stack: target.thread_stack,
            windows: target.windows,
            checks: target.checks,
            cpu: target.cpu,
            size: None,
            source_comment: true,
        }
    }
}

impl Profile {
    pub(crate) fn target(&self) -> cpl_codegen::Target {
        cpl_codegen::Target {
            stamp: self.stamp.clone(),
            thread_stack: self.thread_stack,
            windows: self.windows,
            checks: self.checks,
            cpu: self.cpu.clone(),
        }
    }
}

/// The C++ file of a program.
pub struct CppFile {
    pub text: String,
    /// What the file left out to fit the profile's size, in the order it
    /// gave way. Empty with no size bar.
    pub left_out: Vec<&'static str>,
    /// How long it is as written, with nothing left out.
    pub full: usize,
    /// Whether it is still over the profile's limit.
    pub over_limit: bool,
    fitted: Option<cpl_codegen::Fitted>,
}

/// Where the bytes of a C++ file go.
pub type Size = cpl_codegen::Size;
/// What a [`Size`] measures.
pub type Part = cpl_codegen::Part;

impl Session {
    /// The C++17 file of `program`, written from the file called `name`,
    /// for the judge `profile` describes. The source opens it as a comment.
    pub fn emit_cpp(&self, program: &Program, name: &str, profile: &Profile) -> CppFile {
        emit_cpp(program, name, profile)
    }
}

/// [`Session::emit_cpp`]: a built program holds all its C++ needs.
pub fn emit_cpp(program: &Program, name: &str, profile: &Profile) -> CppFile {
    let target = profile.target();
    let Some(bar) = profile.size else {
        let text = cpl_codegen::emit_for(
            &program.ir,
            name,
            profile.source_comment.then_some(&*program.text),
            &target,
        );
        return CppFile {
            full: text.len(),
            text,
            left_out: Vec::new(),
            over_limit: false,
            fitted: None,
        };
    };
    let fitted = cpl_codegen::emit_aiming(
        &program.ir,
        name,
        &program.text,
        (bar.aim, bar.limit),
        &target,
    );
    let over_limit = fitted.is_err();
    let fitted = fitted.unwrap_or_else(|over| over);
    CppFile {
        text: fitted.cpp.clone(),
        left_out: fitted.left_out(),
        full: fitted.full,
        over_limit,
        fitted: Some(fitted),
    }
}

impl CppFile {
    /// What was left out to fit, said to the contestant of `file` built
    /// for `judge`, whose size `bar` words; `None` when nothing was.
    pub fn note(&self, file: &str, bar: &str, judge: &str) -> Option<String> {
        self.fitted.as_ref()?.note(file, bar, judge)
    }

    /// Where the bytes of this file, fitted to `profile`'s size from
    /// `program`, go: every piece, adding up to its length.
    pub fn sizes(&self, program: &Program, name: &str, profile: &Profile) -> Vec<Size> {
        let Some(fitted) = &self.fitted else {
            return Vec::new();
        };
        let source = fitted.header.then_some(&*program.text);
        cpl_codegen::sizes(&program.ir, name, source, &profile.target(), fitted)
    }
}

/// The C++ of `program` that counts how often each statement runs and
/// writes the counts to the file `tally` when it ends, and the span of
/// each statement counted, in the order of the counts.
pub fn emit_cpp_profiled(
    program: &Program,
    name: &str,
    profile: &Profile,
    tally: &str,
) -> (String, Vec<cpl_diag::Span>) {
    cpl_codegen::emit_profiled(&program.ir, name, &profile.target(), tally)
}
