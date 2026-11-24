//! What a file's parsed top-level fns ask of a run: a judge for
//! `cpl stress`, or a C++ body only the C++ build runs.

use cpl_syntax::ast;

/// The decorators that give `cpl stress` a judge: `@alt` (a whole
/// program or one part), the older `@brute`, `@check` and the older
/// `@checker`, and `@interactor`.
pub const STRESS_DECORATORS: [&str; 5] = ["alt", "brute", "check", "checker", "interactor"];

/// Whether `source` has a top-level fn `cpl stress` judges by: one of
/// [`STRESS_DECORATORS`] on it, as the parser reads it (not a mention in
/// a comment or a string).
pub fn has_stress_target(source: &str) -> bool {
    let parsed = cpl_syntax::parse_file(source);
    let mut stmts = parsed.items.iter().flat_map(|(_, item)| &item.stmts);
    stmts.any(|stmt| {
        matches!(stmt.kind, ast::StmtKind::Fn(_))
            && (stmt.decorators.iter()).any(|d| STRESS_DECORATORS.contains(&d.name.text.as_str()))
    })
}

/// Whether `source` has a top-level `fn f(..) = @cpp("...")`, which only
/// the C++ build runs (Rbundle.5).
pub fn has_cpp_body(source: &str) -> bool {
    let parsed = cpl_syntax::parse_file(source);
    let mut stmts = parsed.items.iter().flat_map(|(_, item)| &item.stmts);
    stmts.any(|stmt| match &stmt.kind {
        ast::StmtKind::Fn(f) => matches!(&f.body, ast::FnBody::Expr(e) if crate::lower::is_cpp(e)),
        _ => false,
    })
}
