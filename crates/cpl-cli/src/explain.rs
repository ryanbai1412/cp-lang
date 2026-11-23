//! `cpl explain`: an error code's long explanation, or what a file lowers to.

use std::io::Write;

use crate::{commands::EXPLAIN_USAGE, compile, EXIT_USAGE};

pub(crate) fn explain_code(rest: &[&str], stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    match rest {
        ["--json"] => {
            let _ = writeln!(stdout, "[");
            let mut first = true;
            for info in cpl_diag::REGISTRY {
                if !first {
                    let _ = writeln!(stdout, ",");
                }
                first = false;
                let _ = write!(
                    stdout,
                    "  {{\"code\":{},\"title\":{},\"summary\":{},\"explanation\":{},\"trigger\":{},\"fix\":{},\"mode\":{},\"input\":{},\"lint_name\":null}}",
                    cpl_json::escape_string(info.code.0),
                    cpl_json::escape_string(info.title),
                    cpl_json::escape_string(info.summary),
                    cpl_json::escape_string(info.explanation),
                    cpl_json::escape_string(info.trigger),
                    cpl_json::escape_string(info.fix),
                    cpl_json::escape_string(info.mode),
                    cpl_json::escape_string(info.input),
                );
            }
            for lint in cpl_lint::LINTS {
                if !first {
                    let _ = writeln!(stdout, ",");
                }
                first = false;
                let _ = write!(
                    stdout,
                    "  {{\"code\":{},\"title\":{},\"summary\":{},\"explanation\":{},\"trigger\":{},\"fix\":{},\"mode\":\"lint\",\"lint_name\":{}}}",
                    cpl_json::escape_string(lint.code.0),
                    cpl_json::escape_string(lint.title),
                    cpl_json::escape_string(lint.summary),
                    cpl_json::escape_string(lint.explanation),
                    cpl_json::escape_string(lint.trigger),
                    cpl_json::escape_string(lint.fix),
                    cpl_json::escape_string(lint.name),
                );
            }
            let _ = writeln!(stdout, "\n]");
            0
        }
        [arg] if arg.ends_with(".cp") || std::path::Path::new(arg).is_file() => {
            explain_file(arg, false, stdout, stderr)
        }
        [arg, "--json"] | ["--json", arg]
            if arg.ends_with(".cp") || std::path::Path::new(arg).is_file() =>
        {
            explain_file(arg, true, stdout, stderr)
        }
        [code] => {
            if let Some(info) = cpl_diag::lookup(code) {
                let _ = writeln!(stdout, "{}: {}", info.code.0, info.title);
                let _ = writeln!(stdout, "\n{}", info.explanation);
                if !info.trigger.is_empty() {
                    let _ = writeln!(stdout, "\nTrigger:\n{}", info.trigger.trim_end());
                }
                if !info.fix.is_empty() {
                    let _ = writeln!(stdout, "\nFix:\n{}", info.fix.trim_end());
                }
                return 0;
            }
            if let Some(lint) = cpl_lint::LINTS
                .iter()
                .find(|l| l.code.0.eq_ignore_ascii_case(code) || l.name == *code)
            {
                let plain = cpl_lint::plain_math;
                let (title, text) = (plain(lint.title), plain(lint.explanation));
                let _ = writeln!(stdout, "{} ({}): {title}", lint.code.0, lint.name);
                let _ = writeln!(stdout, "\n{text}");
                let _ = writeln!(stdout, "\nTrigger:\n{}", lint.trigger.trim_end());
                let _ = writeln!(stdout, "\nFix:\n{}", lint.fix.trim_end());
                return 0;
            }
            *stderr += &format!(
                "cpl: `{code}` is not a diagnostic or lint code, a lint name, or a file\n\n{}",
                EXPLAIN_USAGE
            );
            EXIT_USAGE
        }
        _ => {
            *stderr += &format!(
                "cpl: `explain` takes a code or a file: `cpl explain E0302`\n\n{}",
                EXPLAIN_USAGE
            );
            EXIT_USAGE
        }
    }
}

pub(crate) fn explain_file(
    path: &str,
    json: bool,
    stdout: &mut dyn Write,
    stderr: &mut String,
) -> u8 {
    let compiled = match compile(path, stderr) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let entries = cpl_codegen::explain_closures(&compiled.program);
    let dropped = cpl_codegen::explain_dropped_fields(&compiled.program);
    const DROPPED: &str = "never read; it and its stores are left out";
    let mut db = cpl_check::Database::new();
    db.set_file(path, compiled.text.as_str());
    if json {
        let row = |site: &str, lowering: &str, detail: &str| {
            format!(
                "  {{\"site\":{},\"lowering\":{},\"detail\":{}}}",
                cpl_json::escape_string(site),
                cpl_json::escape_string(lowering),
                cpl_json::escape_string(detail),
            )
        };
        let mut rows = Vec::new();
        for entry in &entries {
            let (kind, detail) = match &entry.mode {
                cpl_codegen::ClosureMode::StaticDirect { closure } => {
                    ("static_direct", format!("cpl_fn{closure}"))
                }
                cpl_codegen::ClosureMode::StaticTemplate => {
                    ("static_template", "template functor parameter".to_string())
                }
                cpl_codegen::ClosureMode::StaticLocalAuto => {
                    ("static_local_auto", "local auto lambda".to_string())
                }
                cpl_codegen::ClosureMode::Unread => ("unread", "never read, dropped".to_string()),
                cpl_codegen::ClosureMode::DynamicStdFunction { reason } => {
                    ("dynamic_std_function", reason.clone())
                }
            };
            rows.push(row(&entry.site, kind, &detail));
        }
        for site in &dropped {
            rows.push(row(site, "dropped_field", DROPPED));
        }
        for (span, what, why) in cpl_codegen::explain_copies(&compiled.program) {
            let (line, col) = cpl_codegen::explain::line_col(&compiled.text, span);
            rows.push(row(
                &format!("{line}:{col}"),
                "copy",
                &format!("{what}: {why}"),
            ));
        }
        for (name, rep) in cpl_lsp::explain::storage(&mut db, path).0 {
            let detail = format!("{}: {}", rep.ty, cpl_lsp::explain::describe(&rep));
            rows.push(row(&name, "storage", &detail));
        }
        let _ = writeln!(stdout, "[\n{}\n]", rows.join(",\n"));
        return 0;
    }
    let section = cpl_lsp::explain::storage_section(&mut db, path);
    let _ = write!(
        stdout,
        "{}",
        cpl_codegen::explain::report_with(path, &compiled.text, &compiled.program, Some(&section))
    );
    0
}
