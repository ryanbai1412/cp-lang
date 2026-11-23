//! `--json` output: what tools read instead of the text a person reads.

use cpl_diag::LineIndex;
use cpl_json::{obj, Value};

pub(crate) use cpl_diag::json::diagnostic;

fn text(s: &str) -> Value {
    Value::String(s.to_string())
}

/// `cpl lint`, with `--json` printing the `files` shape `cpl check --json` prints.
pub(crate) fn lint(
    args: &[String],
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> u8 {
    cpl_lint::cli::run_with(
        args,
        stdout,
        stderr,
        Some(&lint_files),
        Some(&|analysis: &cpl_check::Analysis| cpl_ir::reported(analysis)),
        Some(&save_witnesses),
    )
}

/// `cpl lint --save`: each witness pinned into the program's reads, the
/// rest of the input made by `cpl stress`'s generator, saved as
/// `<file>.tests/lint-<code>-<line>.in` for `cpl test`.
fn save_witnesses(
    file: &std::path::Path,
    src: &str,
    diagnostics: &[cpl_diag::Diagnostic],
) -> Vec<String> {
    let lines = LineIndex::new(src);
    let mut out = Vec::new();
    for d in diagnostics.iter().filter(|d| !d.witness.is_empty()) {
        let line = lines.line_col(d.span.start).line + 1;
        let named: Vec<String> = d
            .witness
            .iter()
            .map(|(n, v)| format!("{n} = {v}"))
            .collect();
        let what = format!(
            "`{}`, the input {} at line {line} names,",
            named.join(", "),
            d.code.0
        );
        let mut spec = match cpl_stress::InputSpec::from_source(src) {
            Ok(spec) => spec,
            Err(e) => {
                out.push(format!("note: {what} is not saved: {}", e.message));
                continue;
            }
        };
        if let Some(w) = d.witness.iter().find(|w| !spec.pin(&w.0, w.1)) {
            out.push(format!(
                "note: {what} is not saved: the program does not read `{}` as one integer",
                w.0
            ));
            continue;
        }
        let mut rng = cpl_stress::Rng::new(1);
        let Some(case) = cpl_stress::generate(&spec, &mut rng, cpl_stress::Size::Small(5)) else {
            out.push(format!(
                "note: {what} is not saved: no input fits the other declared reads"
            ));
            continue;
        };
        let dir = file.with_extension("tests");
        let path = dir.join(format!("lint-{}-{line}.in", d.code.0));
        match std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, case.render())) {
            Ok(()) => out.push(format!(
                "saved {what} as {}; `cpl test` runs it",
                path.display()
            )),
            Err(e) => out.push(format!("error: can't write {}: {e}", path.display())),
        }
    }
    out
}

/// `cpl fmt`, with `--json` printing the shared diagnostic shape.
pub(crate) fn fmt(
    args: &[String],
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> u8 {
    cpl_fmt::cli::run_with(args, stdout, stderr, Some(&fmt_files))
}

fn fmt_files(files: &[cpl_fmt::cli::FileReport]) -> String {
    let files = files.iter().map(|f| {
        let lines = LineIndex::new(&f.src);
        let diagnostics = f.diagnostics.iter().map(|d| diagnostic(d, &lines));
        obj([
            ("path", text(&f.path)),
            ("changed", Value::Bool(f.changed)),
            ("ok", Value::Bool(f.ok)),
            ("diagnostics", Value::Array(diagnostics.collect())),
            ("error", f.error.as_deref().map_or(Value::Null, text)),
        ])
    });
    cpl_json::to_string(&obj([("files", Value::Array(files.collect()))]))
}

fn lint_files(files: &[cpl_lint::cli::FileReport]) -> String {
    let files = files.iter().map(|f| {
        let lines = LineIndex::new(&f.src);
        let diagnostics = f.diagnostics.iter().map(|d| diagnostic(d, &lines));
        obj([
            ("path", text(&f.path)),
            ("ok", Value::Bool(f.ok)),
            ("diagnostics", Value::Array(diagnostics.collect())),
        ])
    });
    cpl_json::to_string(&obj([("files", Value::Array(files.collect()))]))
}
