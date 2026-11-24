//! `cpl explain <file>` as text: how each closure, view and copy lowers to
//! C++. `cpl explain` prints it and the playground shows it.

use crate::ClosureMode;
use std::fmt::Write;

const DROPPED: &str = "never read; it and its stores are left out";
const SUNK: &str = "the list passed here is never built; its loop runs at each push";
const HASHED: &str = "a hash table: nothing reads its order";

/// Whether a copy's reason says a move or a borrow in its place would
/// show ([`crate::uses::KEPT`]): `detail` ends with that reason.
pub fn kept_copy(detail: &str) -> bool {
    crate::uses::KEPT.iter().any(|why| detail.ends_with(why))
}

/// The report for `program`, lowered from `text`, naming the file `path`.
pub fn report(path: &str, text: &str, program: &cpl_ir::Program) -> String {
    report_with(path, text, program, None)
}

/// [`report`] opening with `storage`, the types and memory section the
/// checker's representation query fills (`cpl explain` and the playground
/// Why a `mut` argument is passed as a copy.
pub const WRITTEN_BACK: &str =
    "a `Bool` element is a bit with no address: passed as a copy, put back after the call";

/// always pass it).
pub fn report_with(
    path: &str,
    text: &str,
    program: &cpl_ir::Program,
    storage: Option<&str>,
) -> String {
    let mut out = storage.unwrap_or_default().to_string();
    let stdout = &mut out;
    let crate::Lowered {
        closures: entries,
        copies,
        dropped,
        specializations,
        checks,
        overflows,
    } = crate::explain_lowered(program);
    let view_ops = explain_views(text, program);
    let sunk = explain_sunk(text, program);
    let hashed = explain_hashed(program);
    if !copies.is_empty() {
        let _ = writeln!(
            stdout,
            "Copies for {path} (judge build; every other heap read is a reference):"
        );
        for (span, what, why) in &copies {
            let (line, col) = line_col(text, *span);
            let shown = match what.is_empty() {
                true => text
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or("")
                    .to_string(),
                false => what.clone(),
            };
            let _ = writeln!(stdout, "  [copy] {line}:{col}  {shown} -> {why}");
        }
    }
    if !program.written_back.is_empty() {
        let _ = writeln!(stdout, "Written back for {path}:");
        for span in &program.written_back {
            let (line, col) = line_col(text, *span);
            let shown = text
                .get(span.start as usize..span.end as usize)
                .unwrap_or("");
            let _ = writeln!(
                stdout,
                "  [written back] {line}:{col}  {shown} -> {WRITTEN_BACK}"
            );
        }
    }
    if !hashed.is_empty() {
        let _ = writeln!(stdout, "Tables stored as hash tables for {path}:");
        for ty in &hashed {
            let _ = writeln!(stdout, "  [hashed] {ty} -> {HASHED}");
        }
    }
    if !program.cpp_bodies.is_empty() {
        let _ = writeln!(
            stdout,
            "C++ bodies for {path} (`@cpp`, emitted as written; `cpl run` runs the C++ build):"
        );
        for (id, _) in &program.cpp_bodies {
            let _ = writeln!(stdout, "  [cpp] {}", program.fns[*id].name);
        }
    }
    if !sunk.is_empty() {
        let _ = writeln!(stdout, "Reductions sunk for {path} (judge build):");
        for (line, col, call) in &sunk {
            let _ = writeln!(stdout, "  [sunk] {line}:{col}  {call}(..) -> {SUNK}");
        }
    }
    if !dropped.is_empty() {
        let _ = writeln!(stdout, "Dropped fields for {path}:");
        for site in &dropped {
            let _ = writeln!(stdout, "  [dropped] {site} -> {DROPPED}");
        }
    }
    if entries.is_empty() && view_ops.is_empty() {
        if dropped.is_empty()
            && sunk.is_empty()
            && hashed.is_empty()
            && copies.is_empty()
            && program.written_back.is_empty()
        {
            let _ = writeln!(stdout, "{path}: no closure or fn-typed sites");
        }
        chosen_and_checked(stdout, path, text, &specializations, &checks);
        overflow_checked(stdout, path, text, &overflows);
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
                        "  [static: template] {} -> template param (no std::function)",
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
                ClosureMode::Unbuilt => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: unbuilt]  {} -> its struct is never built, never called (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::StaticFunctor { closures } => {
                    static_count += 1;
                    let (kind, what) = match closures.len() {
                        1 => ("type", "its one closure's own type".to_string()),
                        n => ("variant", format!("a variant over its {n} closures' own types, called through a switch")),
                    };
                    let _ = writeln!(
                        stdout,
                        "  [static: closure {kind}] {} -> {} ({what}, no std::function)",
                        entry.site,
                        crate::devirt::functor_name(closures)
                    );
                }
                ClosureMode::FnPointer => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: fn ptr]   {} -> every closure it can hold captures nothing (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::Bound => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: bound]    {} -> every closure it can hold copies captures of the same types (no std::function)",
                        entry.site
                    );
                }
                ClosureMode::FnRef => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: fn ref]   {} -> a view of the caller's callable, a pointer and a trampoline (no std::function)",
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
        let _ = writeln!(stdout, "Views & concatenations:");
        for (line, col, desc) in view_ops {
            let _ = writeln!(stdout, "  {line}:{col}  {desc}");
        }
    }
    chosen_and_checked(stdout, path, text, &specializations, &checks);
    overflow_checked(stdout, path, text, &overflows);
    out
}

/// What the C++ does at one index the file writes, in a build that keeps
/// index checks (`--checks traps` or `all`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Checked,
    /// The index's type keeps it in range: a domain's position.
    Typed,
    /// One check before the loop stands for every round's.
    Hoisted,
    /// A raw index, which no build checks.
    Raw,
}

impl Index {
    /// Why the index has no check of its own; empty when it has one.
    pub fn why(self) -> &'static str {
        match self {
            Index::Checked => "",
            Index::Typed => "its index's type keeps it in range",
            Index::Hoisted => "one check before its loop covers it",
            Index::Raw => "a raw index, never checked",
        }
    }
}

/// Why an `Int` `+ - *` has no overflow check.
pub const FITS: &str = "its operands' proven ranges keep it in `Int`";

/// Each `Int` `+ - *` with whether the checked builds test it for overflow,
/// as the shared range facts decide; nothing when the file has none.
fn overflow_checked(stdout: &mut String, path: &str, text: &str, ops: &[(cpl_diag::Span, bool)]) {
    if ops.is_empty() {
        return;
    }
    let _ = writeln!(
        stdout,
        "Overflow checks left per build for {path}, by `Int` `+ - *` (`--checks none` keeps none; `traps` and `all` keep each [check]):"
    );
    for &(span, checked) in ops {
        let (line, col) = line_col(text, span);
        let shown = text
            .get(span.start as usize..span.end as usize)
            .and_then(|s| s.lines().next())
            .unwrap_or("");
        let _ = match checked {
            true => writeln!(stdout, "  [check] {line}:{col}  {shown}"),
            false => writeln!(stdout, "  [no check] {line}:{col}  {shown} -> {FITS}"),
        };
    }
}

/// The specializations the C++ takes and the bound behind each, and each
/// index with whether the checked builds check it.
fn chosen_and_checked(
    stdout: &mut String,
    path: &str,
    text: &str,
    specializations: &[String],
    checks: &[(cpl_diag::Span, Index)],
) {
    if specializations.is_empty() {
        let _ = writeln!(stdout, "Specializations chosen for {path}: none");
    } else {
        let _ = writeln!(stdout, "Specializations chosen for {path}:");
        for chosen in specializations {
            let _ = writeln!(stdout, "  [chosen] {chosen}");
        }
    }
    if checks.is_empty() {
        let _ = writeln!(stdout, "Checks left per build for {path}: no index");
        return;
    }
    let _ = writeln!(
        stdout,
        "Checks left per build for {path}, by index (`--checks none`, the judge default, keeps none; `traps` and `all` keep each [check]):"
    );
    for (span, index) in checks {
        let (line, col) = line_col(text, *span);
        let shown = text
            .get(span.start as usize..span.end as usize)
            .and_then(|s| s.lines().next())
            .unwrap_or("");
        let _ = match index {
            Index::Checked => writeln!(stdout, "  [check] {line}:{col}  {shown}"),
            _ => writeln!(
                stdout,
                "  [no check] {line}:{col}  {shown} -> {}",
                index.why()
            ),
        };
    }
}

/// Each `Map` and `Set` type emitted as a hash table, as the source writes it.
fn explain_hashed(program: &cpl_ir::Program) -> Vec<String> {
    let (maps, sets) = crate::unordered::find(program);
    let any = cpl_ir::Ty::Int;
    let mut shown: Vec<String> = maps
        .iter()
        .map(|k| program.show_ty(&cpl_ir::Ty::Map(Box::new(k.clone()), Box::new(any.clone()))))
        .map(|t| {
            t.rsplit_once(',')
                .map_or(t.clone(), |(head, _)| format!("{head}, _>"))
        })
        .chain(
            sets.iter()
                .map(|e| program.show_ty(&cpl_ir::Ty::Set(Box::new(e.clone())))),
        )
        .collect();
    shown.sort();
    shown.dedup();
    shown
}

/// Each sunk call as its line, column and callee as the source writes it.
fn explain_sunk(text: &str, program: &cpl_ir::Program) -> Vec<(usize, usize, String)> {
    let mut sunk: Vec<_> = crate::sink::explained(program)
        .into_iter()
        .filter_map(|span| {
            let start = span.start as usize;
            let call = text.get(start..span.end as usize)?;
            let head = call.split('(').next()?.trim();
            let (line, col) = line_col(text, span);
            Some((line, col, head.to_string()))
        })
        .collect();
    sunk.sort();
    sunk.dedup();
    sunk
}

/// The 1-based line and column of `span` in `text`.
pub fn line_col(text: &str, span: cpl_diag::Span) -> (usize, usize) {
    let off = (span.start as usize).min(text.len());
    let before = &text[..off];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = before.rfind('\n').map_or(off + 1, |i| off - i);
    (line, col)
}

fn explain_views(text: &str, program: &cpl_ir::Program) -> Vec<(usize, usize, String)> {
    let line_col = |span| line_col(text, span);
    let mut ops: Vec<(usize, usize, String)> = Vec::new();
    let mut visit_expr = |e: &cpl_ir::Expr| {
        if let cpl_ir::ExprKind::MutSlice(..) | cpl_ir::ExprKind::MutLine(..) = &e.kind {
            let elem = match &e.ty {
                cpl_ir::Ty::MutView(t, _) => program.show_ty(t),
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
    crate::uses::split::each_value(&program.setup, &mut visit_expr);
    crate::uses::split::each_value(&program.main, &mut visit_expr);
    // Library bodies' spans index their own files, not `text`.
    for (id, f) in program.fns.iter().enumerate() {
        if !program.library_fns.get(id).copied().unwrap_or(false) {
            crate::uses::split::each_value(&f.body, &mut visit_expr);
        }
    }
    ops.sort_by_key(|(l, c, _)| (*l, *c));
    ops.dedup();
    ops
}
