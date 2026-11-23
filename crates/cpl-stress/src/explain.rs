//! Why a solution failed at run time, in the words of `cpl run`.
//!
//! The built C++ stops with a signal or an exit code and no message. Run
//! on the same input in the debug interpreter, a `.cp` solution says what
//! went wrong and where, as an `R0001` with the source line.

use cpl_diag::Shown;
use std::path::Path;

/// The runtime error the `.cp` file at `path` stops with on `input`, as
/// `cpl run` would print it. `None` when the file is not CP Lang, does not
/// build, or runs cleanly in the interpreter.
pub fn runtime_error(path: &Path, input: &str) -> Option<String> {
    if path.extension().is_none_or(|ext| ext != "cp") {
        return None;
    }
    let name = path.shown();
    let text = std::fs::read_to_string(path).ok()?;
    let mut db = cpl_check::Database::new();
    db.set_file(&name, text.as_str());
    let analysis = db.analyze(&name)?;
    if analysis
        .diagnostics
        .iter()
        .any(|d| d.severity == cpl_diag::Severity::Error)
    {
        return None;
    }
    let program = cpl_ir::lower(&analysis).ok()?;
    // Alone, an interactive solution only ever sees its input end.
    if program.interactive {
        return None;
    }
    let mut sink = Vec::new();
    let trap = cpl_interp::run(&program, input.as_bytes(), &mut sink).err()?;
    let lines = cpl_diag::LineIndex::new(&text);
    let diagnostic = cpl_diag::Diagnostic::error(trap.code(), trap.span, trap.message);
    Some(cpl_diag::render(&diagnostic, &name, &text, &lines))
}
