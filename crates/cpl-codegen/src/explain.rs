//! `cpl explain <file>` as text: how each closure, view and copy lowers to
//! C++. `cpl explain` prints it and the playground shows it.

use crate::{explain_closures, explain_dropped_fields, ClosureMode};
use std::fmt::Write;

const DROPPED: &str = "never read; it and its stores are left out";

/// The report for `program`, lowered from `text`, naming the file `path`.
pub fn report(path: &str, text: &str, program: &cpl_ir::Program) -> String {
    let mut out = String::new();
    let stdout = &mut out;
    let entries = explain_closures(program);
    let dropped = explain_dropped_fields(program);
    let view_ops = explain_views(text, program);
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
        return out;
    }
    if !entries.is_empty() {
        let mut static_count = 0usize;
        let mut dynamic_count = 0usize;
        let _ = writeln!(stdout, "Closure & operation lowering for {path}:");
        for entry in &entries {
            match &entry.mode {
                ClosureMode::StaticDirect { closure } => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: direct]   {} -> cpl_fn{closure} (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::StaticTemplate => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: template] {} -> template functor param (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::StaticLocalAuto => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: auto]     {} -> local auto lambda (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::Unread => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: dropped]  {} -> never read, dropped (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::DynamicStdFunction { reason } => {
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
    out
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

fn not_shown_yet(stdout: &mut String) {
    let _ = writeln!(stdout, "Not shown yet:");
    for section in NOT_SHOWN_YET {
        let _ = writeln!(stdout, "  {section}");
    }
}

fn explain_views(text: &str, program: &cpl_ir::Program) -> Vec<(usize, usize, String)> {
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
