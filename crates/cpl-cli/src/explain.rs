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
                    "  {{\"code\":{},\"title\":{},\"summary\":{},\"explanation\":{},\"trigger\":{},\"fix\":{},\"mode\":\"lint\",\"lint_name\":{},\"rule\":{}}}",
                    cpl_json::escape_string(lint.code.0),
                    cpl_json::escape_string(lint.title),
                    cpl_json::escape_string(lint.summary),
                    cpl_json::escape_string(lint.explanation),
                    cpl_json::escape_string(lint.trigger),
                    cpl_json::escape_string(lint.fix),
                    cpl_json::escape_string(lint.name),
                    lint.rule.map_or("null".to_string(), cpl_json::escape_string),
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
                if let Some(rule) = lint.rule {
                    let topic = rule.split_once('/').map_or(rule, |(topic, _)| topic);
                    let _ = writeln!(
                        stdout,
                        "\nRule: {rule} (handbook/guidelines/{topic}-checklist.md)"
                    );
                }
                return 0;
            }
            if explain_library_lint(code, stdout) {
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
                "cpl: `explain` takes a code or a file: `cpl explain {}`\n\n{}",
                cpl_diag::codes::TYPE_MISMATCH.0,
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
    if json {
        let entries = cpl_codegen::explain_closures(&compiled.program);
        let dropped = cpl_codegen::explain_dropped_fields(&compiled.program);
        const DROPPED: &str = "never read; it and its stores are left out";
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
                cpl_codegen::ClosureMode::Unbuilt => (
                    "unbuilt",
                    "its struct is never built, never called".to_string(),
                ),
                cpl_codegen::ClosureMode::FnPointer => (
                    "fn_pointer",
                    "every closure it can hold captures nothing".to_string(),
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
        for (span, what, why) in cpl_codegen::explain_copies(&compiled.program) {
            let (line, col) = cpl_codegen::explain::line_col(&compiled.text, span);
            rows.push(row(
                &format!("{line}:{col}"),
                "copy",
                &format!("{what}: {why}"),
            ));
        }
        for (name, rep) in crate::with_db(|db| cpl_lsp::explain::storage(db, path)).0 {
            let detail = format!("{}: {}", rep.ty, cpl_lsp::explain::describe(&rep));
            rows.push(row(&name, "storage", &detail));
        }
        let _ = writeln!(stdout, "[\n{}\n]", rows.join(",\n"));
        return 0;
    }
    let section = crate::with_db(|db| cpl_lsp::explain::storage_section(db, path));
    let _ = write!(
        stdout,
        "{}",
        cpl_codegen::explain::report_with(path, &compiled.text, &compiled.program, Some(&section))
    );
    0
}

/// `cpl explain <name>` for a lint a library declares with `@lint`
/// (R06.4.9): found in the libraries the `cp.toml` files at or above the
/// current directory name under `[libs]`.
fn explain_library_lint(name: &str, stdout: &mut dyn Write) -> bool {
    let cwd = std::env::current_dir().unwrap_or_default();
    for (lib, root) in cpl_check::configured_libraries(&cwd) {
        let mut files = Vec::new();
        cp_files(&root, &mut files);
        files.sort();
        for file in files {
            let Ok(src) = std::fs::read_to_string(&file) else {
                continue;
            };
            let parsed = cpl_syntax::parse_file(&src);
            for (offset, item) in &parsed.items {
                for stmt in &item.stmts {
                    let decls = cpl_syntax::library_lint::declared(stmt, *offset).0;
                    let Some(decl) = decls.into_iter().find(|d| d.name == name) else {
                        continue;
                    };
                    let rel = file.strip_prefix(&root).unwrap_or(&file).with_extension("");
                    let module = std::iter::once(lib.clone())
                        .chain(rel.iter().map(|c| c.to_string_lossy().into_owned()))
                        .collect::<Vec<_>>()
                        .join(".");
                    let line = src[..decl.decorator.start as usize].matches('\n').count() + 1;
                    let _ = writeln!(
                        stdout,
                        "{name}: a lint `{module}` declares with `@lint` on `{}`",
                        decl.fn_name
                    );
                    let _ = writeln!(
                        stdout,
                        "\nIt fires where code writes `{}`, and suggests `{}`.",
                        decl.from, decl.to
                    );
                    let _ = writeln!(
                        stdout,
                        "\nFix:\n`cpl fix` writes the call. `with lint({name}: allow):` keeps a block as written; `{name} = \"allow\"` under `[lints]` in cp.toml turns it off."
                    );
                    let _ = writeln!(stdout, "\nDeclared at {}:{line}", file.display());
                    return true;
                }
            }
        }
    }
    false
}

/// The `.cp` files below `dir`.
fn cp_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            cp_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}
