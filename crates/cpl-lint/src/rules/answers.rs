//! `bool_ternary_print` (L0524): `if ok: "YES" else: "NO"` spells out the
//! answer words that `yes(ok)` takes from the judge profile or
//! `with io(answers: ...)`.
//!
//! No fix: which words `yes` prints depends on the profile, which the
//! linter does not see, so the rewrite is the contestant's to make.

use crate::walk::Visitor;
use crate::{File, Finding, BOOL_TERNARY_PRINT};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{Block, Cond, Expr, ExprKind, StmtKind, StrPiece};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Answers<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Answers<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::If {
                branches,
                otherwise: Some(no),
            } = &expr.kind
            else {
                return;
            };
            let [(Cond::Expr(cond), yes)] = &branches[..] else {
                return;
            };
            let (Some((t, printed)), Some((f, also))) = (word(yes), word(no)) else {
                return;
            };
            if printed != also {
                return;
            }
            if !t.eq_ignore_ascii_case("YES") || !f.eq_ignore_ascii_case("NO") {
                return;
            }
            let cond = cond.span.shift(self.base).text(self.src);
            let call = if printed {
                format!("print(yes({cond}))")
            } else {
                format!("yes({cond})")
            };
            let help = if (t, f) == ("YES", "NO") {
                format!("write `{call}`; `YES` and `NO` are the default words")
            } else {
                format!(
                    "write `{call}`, and set the words once with \
                     `with io(answers: (\"{t}\", \"{f}\"))` or the judge profile"
                )
            };
            let diag = Diagnostic::warning(
                BOOL_TERNARY_PRINT.code,
                expr.span.shift(self.base),
                format!("`\"{t}\"` and `\"{f}\"` written out; `yes({cond})` is the answer word"),
            )
            .with_help(help);
            self.out.push(Finding {
                lint: &BOOL_TERNARY_PRINT,
                diag,
            });
        }
    }
    let mut v = Answers {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// The text of a block that is one plain string literal, or one `print` of
/// it (then `true`).
fn word(block: &Block) -> Option<(&str, bool)> {
    let [stmt] = &block.stmts[..] else {
        return None;
    };
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    let (e, printed) = match &e.kind {
        ExprKind::Call { callee, args }
            if matches!(&callee.kind, ExprKind::Name(f) if f == "print") && args.len() == 1 =>
        {
            (&args[0].value, true)
        }
        _ => (e, false),
    };
    match &e.kind {
        ExprKind::Str(pieces) => match &pieces[..] {
            [StrPiece::Text(t)] => Some((t, printed)),
            _ => None,
        },
        _ => None,
    }
}
