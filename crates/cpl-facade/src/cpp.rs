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
            checks: target.checks.into(),
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
            checks: self.checks.into(),
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

/// The C++ declarations of the fns that cross to the judge's grader, as a
/// header that stands alone, from a program's C++ (R10.1.3); `None` when
/// the program has no grader.
pub fn grader_header(cpp: &str) -> Option<String> {
    let (open, close) = cpl_ir::GRADER_BOUNDARY;
    let (_, rest) = cpp.split_once(open)?;
    let (decls, _) = rest.split_once(close)?;
    Some(format!(
        "#include <string>\n#include <tuple>\n#include <utility>\n#include <vector>\n{decls}"
    ))
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

/// One thing the C++ of a program does that its source does not spell: how
/// a closure is called, a copy, a check kept or dropped.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Choice {
    /// Where: `line:col`, a name, or empty for the whole program.
    pub site: String,
    /// Which choice (`static_direct`, `copy`, `index_checked`).
    pub kind: &'static str,
    /// What was chosen, and why where there is a reason.
    pub detail: String,
}

impl Program {
    /// Every choice the C++ of this program, built from `text`, makes.
    pub fn choices(&self, text: &str) -> Vec<Choice> {
        let entries = cpl_codegen::explain_closures(&self.ir);
        let dropped = cpl_codegen::explain_dropped_fields(&self.ir);
        const DROPPED: &str = "never read; it and its stores are left out";
        let row = |site: &str, kind: &'static str, detail: &str| Choice {
            site: site.to_string(),
            kind,
            detail: detail.to_string(),
        };
        let mut rows = Vec::new();
        for entry in &entries {
            let (kind, detail) = match &entry.mode {
                cpl_codegen::ClosureMode::StaticDirect { closure } => {
                    ("static_direct", format!("cpl_fn{closure}"))
                }
                cpl_codegen::ClosureMode::StaticTemplate => {
                    ("static_template", "template parameter".to_string())
                }
                cpl_codegen::ClosureMode::StaticLocalAuto => {
                    ("static_local_auto", "local auto lambda".to_string())
                }
                cpl_codegen::ClosureMode::StaticFunctor { closures } => match closures.as_slice() {
                    [closure] => (
                        "static_closure_type",
                        format!("cpl_cl{closure}, its one closure's own type"),
                    ),
                    _ => {
                        let ids: Vec<String> = closures.iter().map(ToString::to_string).collect();
                        (
                            "static_closure_variant",
                            format!(
                                "cpl_cl{}, a variant over its {} closures' own types",
                                ids.join("_"),
                                ids.len()
                            ),
                        )
                    }
                },
                cpl_codegen::ClosureMode::Unread => ("unread", "never read, dropped".to_string()),
                cpl_codegen::ClosureMode::Unbuilt => (
                    "unbuilt",
                    "its struct is never built, never called".to_string(),
                ),
                cpl_codegen::ClosureMode::FnPointer => (
                    "fn_pointer",
                    "every closure it can hold captures nothing".to_string(),
                ),
                cpl_codegen::ClosureMode::Bound => (
                    "bound",
                    "every closure it can hold copies captures of the same types".to_string(),
                ),
                cpl_codegen::ClosureMode::FnRef => (
                    "fn_ref",
                    "a view of the caller's callable, a pointer and a trampoline".to_string(),
                ),
                cpl_codegen::ClosureMode::DynamicStdFunction { reason } => {
                    ("dynamic_std_function", reason.clone())
                }
            };
            rows.push(row(&entry.site, kind, &detail));
        }
        for site in &dropped {
            rows.push(row(site, "dropped_field", DROPPED));
        }
        let lowered = cpl_codegen::explain_lowered(&self.ir);
        for (span, what, why) in &lowered.copies {
            let (line, col) = cpl_codegen::explain::line_col(text, *span);
            rows.push(row(
                &format!("{line}:{col}"),
                "copy",
                &format!("{what}: {why}"),
            ));
        }
        for span in &lowered.moved {
            let (line, col) = cpl_codegen::explain::line_col(text, *span);
            let detail = cpl_codegen::explain::MOVED;
            rows.push(row(&format!("{line}:{col}"), "copy_moved", detail));
        }
        for span in self.written_back() {
            let (line, col) = cpl_codegen::explain::line_col(text, *span);
            let detail = cpl_codegen::explain::WRITTEN_BACK;
            rows.push(row(&format!("{line}:{col}"), "written_back", detail));
        }
        for chosen in &lowered.specializations {
            rows.push(row("", "specialization", chosen));
        }
        for (span, index) in &lowered.checks {
            let (line, col) = cpl_codegen::explain::line_col(text, *span);
            let lowering = match index {
                cpl_codegen::explain::Index::Checked => "index_checked",
                _ => "index_unchecked",
            };
            rows.push(row(&format!("{line}:{col}"), lowering, index.why()));
        }
        for (span, what) in &lowered.proven {
            let (line, col) = cpl_codegen::explain::line_col(text, *span);
            let why = format!("{what}: {}", cpl_codegen::explain::PROVEN);
            rows.push(row(&format!("{line}:{col}"), "check_dropped", &why));
        }
        for &(span, checked) in &lowered.overflows {
            let (line, col) = cpl_codegen::explain::line_col(text, span);
            let (lowering, why) = match checked {
                true => ("overflow_checked", ""),
                false => ("overflow_unchecked", cpl_codegen::explain::FITS),
            };
            rows.push(row(&format!("{line}:{col}"), lowering, why));
        }
        rows
    }

    /// The choices as a report for a person, of the file `path` with
    /// `text`, after what `storage` says of its memory.
    pub fn explained(&self, path: &str, text: &str, storage: Option<&str>) -> String {
        cpl_codegen::explain::report_with(path, text, &self.ir, storage)
    }
}
