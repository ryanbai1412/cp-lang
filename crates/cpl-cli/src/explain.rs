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
    if json {
        let _ = writeln!(stdout, "[");
        for (i, entry) in entries.iter().enumerate() {
            let comma = if i + 1 == entries.len() && dropped.is_empty() {
                ""
            } else {
                ","
            };
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
            let _ = writeln!(
                stdout,
                "  {{\"site\":{},\"lowering\":{},\"detail\":{}}}{comma}",
                cpl_json::escape_string(&entry.site),
                cpl_json::escape_string(kind),
                cpl_json::escape_string(&detail),
            );
        }
        for (i, site) in dropped.iter().enumerate() {
            let comma = if i + 1 == dropped.len() { "" } else { "," };
            let _ = writeln!(
                stdout,
                "  {{\"site\":{},\"lowering\":\"dropped_field\",\"detail\":{}}}{comma}",
                cpl_json::escape_string(site),
                cpl_json::escape_string(DROPPED),
            );
        }
        let _ = writeln!(stdout, "]");
        return 0;
    }
    let view_ops = explain_views(&compiled.text, &compiled.program);
    if !dropped.is_empty() {
        let _ = writeln!(stdout, "Dropped fields for {path}:");
        for site in &dropped {
            let _ = writeln!(stdout, "  [dropped] {site} -> {DROPPED}");
        }
    }
    if entries.is_empty() && view_ops.is_empty() {
        if dropped.is_empty() {
            let _ = writeln!(stdout, "{path}: no closure or fn-typed sites");
        }
        not_shown_yet(stdout);
        return 0;
    }
    if !entries.is_empty() {
        let mut static_count = 0usize;
        let mut dynamic_count = 0usize;
        let _ = writeln!(stdout, "Closure & operation lowering for {path}:");
        for entry in &entries {
            match &entry.mode {
                cpl_codegen::ClosureMode::StaticDirect { closure } => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: direct]   {} -> cpl_fn{closure} (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::StaticTemplate => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: template] {} -> template functor param (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::StaticLocalAuto => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: auto]     {} -> local auto lambda (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::Unread => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: dropped]  {} -> never read, dropped (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::DynamicStdFunction { reason } => {
                    dynamic_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [dynamic: std::function] {} ({reason})",
                        entry.site
                    );
                }
            }
        }
        let _ = writeln!(
            stdout,
            "Summary: {static_count} static (inlinable), {dynamic_count} dynamic (std::function)"
        );
    }
    if !view_ops.is_empty() {
        let _ = writeln!(stdout, "Views & copies:");
        for (line, col, desc) in view_ops {
            let _ = writeln!(stdout, "  {line}:{col}  {desc}");
        }
    }
    not_shown_yet(stdout);
    0
}

/// The sections `spec/tools/explain.md` promises that need the compiler's
/// representation query (plan/QUESTIONS.md tools-3), named so nobody
/// mistakes their absence for "nothing to report".
const NOT_SHOWN_YET: &[&str] = &[
    "types: bytes and niches",
    "arrays and peak memory",
    "specializations chosen",
    "checks left per build",
];

fn not_shown_yet(stdout: &mut dyn Write) {
    let _ = writeln!(stdout, "Not shown yet:");
    for section in NOT_SHOWN_YET {
        let _ = writeln!(stdout, "  {section}");
    }
}

pub(crate) fn explain_views(text: &str, program: &cpl_ir::Program) -> Vec<(usize, usize, String)> {
    let line_col = |span: cpl_diag::Span| -> (usize, usize) {
        let off = (span.start as usize).min(text.len());
        let before = &text[..off];
        let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
        let col = before.rfind('\n').map_or(off + 1, |i| off - i);
        (line, col)
    };
    let mut ops: Vec<(usize, usize, String)> = Vec::new();
    let mut visit_expr = |e: &cpl_ir::Expr| {
        if let cpl_ir::ExprKind::MutSlice(..) = &e.kind {
            let elem = match &e.ty {
                cpl_ir::Ty::MutView(t) => program.show_ty(t),
                _ => "T".to_string(),
            };
            let (line, col) = line_col(e.span);
            ops.push((
                line,
                col,
                format!(
                    "mutable slice view -> cpl::MutView<{elem}> (T* + len), O(1), fixed-length mut lend"
                ),
            ));
        }
        if let cpl_ir::ExprKind::Builtin(op, _) = &e.kind {
            let desc = match op {
                cpl_ir::Builtin::StrSlice => Some(
                    "slice view -> cpl::StrView (std::string_view + rev), O(1), read-only"
                        .to_string(),
                ),
                cpl_ir::Builtin::ListSlice => Some(format!(
                    "slice view -> cpl::ListView<{}> (const T* + len + rev), O(1), read-only",
                    match &e.ty {
                        cpl_ir::Ty::ListView(t, _) | cpl_ir::Ty::Keyed(t, _) => program.show_ty(t),
                        _ => "T".to_string(),
                    }
                )),
                cpl_ir::Builtin::Reversed if matches!(e.ty, cpl_ir::Ty::StrView) => Some(
                    "reverse view -> cpl::StrView (std::string_view + rev), O(1), read-only"
                        .to_string(),
                ),
                cpl_ir::Builtin::Reversed
                    if matches!(e.ty, cpl_ir::Ty::ListView(_, _) | cpl_ir::Ty::Keyed(_, _)) =>
                {
                    Some(format!(
                        "reverse view -> cpl::ListView<{}> (const T* + len + rev), O(1), read-only",
                        match &e.ty {
                            cpl_ir::Ty::ListView(t, _) | cpl_ir::Ty::Keyed(t, _) =>
                                program.show_ty(t),
                            _ => "T".to_string(),
                        }
                    ))
                }
                cpl_ir::Builtin::ToStr => {
                    Some("explicit copy -> std::string, O(n), owned".to_string())
                }
                cpl_ir::Builtin::ToList => Some(format!(
                    "explicit copy -> std::vector<{}>, O(n), owned",
                    match &e.ty {
                        cpl_ir::Ty::List(t) => program.show_ty(t),
                        _ => "T".to_string(),
                    }
                )),
                cpl_ir::Builtin::StrConcat => {
                    Some("concat -> std::string, O(n + m), owned".to_string())
                }
                cpl_ir::Builtin::ListConcat => Some(format!(
                    "concat -> std::vector<{}>, O(n + m), owned",
                    match &e.ty {
                        cpl_ir::Ty::List(t) => program.show_ty(t),
                        _ => "T".to_string(),
                    }
                )),
                _ => None,
            };
            if let Some(desc) = desc {
                let (line, col) = line_col(e.span);
                ops.push((line, col, desc));
            }
        }
    };
    cpl_ir::walk::deep_exprs(&program.setup, &mut visit_expr);
    cpl_ir::walk::deep_exprs(&program.main, &mut visit_expr);
    for f in &program.fns {
        cpl_ir::walk::deep_exprs(&f.body, &mut visit_expr);
    }
    ops.sort_by_key(|(l, c, _)| (*l, *c));
    ops.dedup();
    ops
}
