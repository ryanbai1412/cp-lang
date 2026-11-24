//! Spellings std removed, met in user code: the error carries the rewrite, so `cpl fix`
//! migrates the call (decisions 456 and 511). A row only matches a name that no longer
//! resolves, so a user's own definition of that name is never rewritten.

use super::*;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

/// A removed call and how it is written now. In `to`, `{0}`, `{1}`, .. are the positional
/// arguments, `{self}` is a method's receiver and `{named}` the named arguments, each written
/// `, name: value`.
pub(crate) struct Removed {
    pub name: &'static str,
    pub method: bool,
    pub arity: usize,
    pub to: &'static str,
    pub why: &'static str,
}

pub(crate) const REMOVED: &[Removed] = &[
    Removed {
        name: "suffix_fold",
        method: false,
        arity: 2,
        to: "{1}.rev.scan({0})",
        why: "a suffix fold is the scan of the reversed view (R07.4.16)",
    },
    Removed {
        name: "bsearch_real",
        method: false,
        arity: 3,
        to: "({0}..{1}).bsearch({2}{named})",
        why: "a real binary search is a method of its range",
    },
    Removed {
        name: "ternary_max_real",
        method: false,
        arity: 3,
        to: "({0}..{1}).peak(by: {2}{named})",
        why: "the peak of a unimodal function is a method of its range",
    },
    Removed {
        name: "ternary_min_real",
        method: false,
        arity: 3,
        to: "({0}..{1}).valley(by: {2}{named})",
        why: "the valley of a unimodal function is a method of its range",
    },
    Removed {
        name: "skip_while",
        method: true,
        arity: 1,
        to: "{self}.drop_while({0})",
        why: "`skip_while` is spelled `drop_while`",
    },
];

/// `prefix_excl` is rewritten by `Checker::prefix_excl_edits`: it hoists two scans.
const PREFIX_EXCL: &str = "prefix_excl";

/// Whether std removed `name`, so a `use` of it goes too.
pub(crate) fn is_removed(name: &str) -> bool {
    name == PREFIX_EXCL || REMOVED.iter().any(|r| r.name == name)
}

/// `to` with its holes filled; a hole followed by `.` gets parentheses unless it is an atom.
fn fill(
    to: &str,
    receiver: Option<(&str, bool)>,
    positional: &[(&str, bool)],
    named: &str,
) -> String {
    let mut out = String::new();
    let mut rest = to;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = open + rest[open..].find('}').unwrap_or(rest.len() - open);
        let key = &rest[open + 1..close];
        rest = rest.get(close + 1..).unwrap_or("");
        let hole = match key {
            "self" => receiver,
            "named" => Some((named, true)),
            k => k
                .parse::<usize>()
                .ok()
                .and_then(|i| positional.get(i).copied()),
        };
        let Some((text, atom)) = hole else {
            continue;
        };
        if !atom && rest.starts_with('.') {
            out.push_str(&format!("({text})"));
        } else {
            out.push_str(text);
        }
    }
    out.push_str(rest);
    out
}

fn is_atom(e: &Expr, text: &str) -> bool {
    matches!(
        e.kind,
        ExprKind::Name(_) | ExprKind::Call { .. } | ExprKind::Field { .. } | ExprKind::Index { .. }
    ) || !text.contains(' ')
        || (text.starts_with('[') && text.ends_with(']'))
}

/// `base`, or `base2`, `base3`, .. : the `skip`-th (from 0) that `src` doesn't use as a word,
/// so the rewrites of several calls in one pass take different names.
fn fresh(src: &str, base: &str, skip: usize) -> String {
    let used = |name: &str| {
        src.match_indices(name).any(|(i, _)| {
            let word = |c: char| c.is_alphanumeric() || c == '_';
            !src[..i].chars().next_back().is_some_and(word)
                && !src[i + name.len()..].chars().next().is_some_and(word)
        })
    };
    (1..)
        .map(|n| match n {
            1 => base.to_string(),
            n => format!("{base}{n}"),
        })
        .filter(|name| !used(name))
        .nth(skip)
        .unwrap_or_default()
}

impl Checker<'_> {
    /// `suffix_fold(op, a)`, `a.prefix_excl(op)`, ..: a removed spelling whose name no longer
    /// resolves. Its error says so and carries the rewrite. True when it matched.
    pub(super) fn removed_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> bool {
        let (name, receiver, at, code) = match &callee.kind {
            ExprKind::Name(n) => (n.as_str(), None, callee.span, codes::UNRESOLVED_NAME),
            ExprKind::Field { base, name } => (
                name.text.as_str(),
                Some(&**base),
                name.span,
                codes::UNKNOWN_FIELD,
            ),
            _ => return false,
        };
        let callee_error = |d: &cpl_diag::Diagnostic| d.code == code && d.span == at;
        let Some(mut diag) = self
            .sink
            .diagnostics()
            .iter()
            .find(|d| callee_error(d))
            .cloned()
        else {
            return false;
        };
        let (ours, edits, why) = if name == PREFIX_EXCL {
            let (ours, edits) = match self.prefix_excl_edits(call, receiver, args) {
                Some((ours, edits)) => (ours, edits),
                None => (String::new(), Vec::new()),
            };
            (
                ours,
                edits,
                "everything but `i` folds the scans before and after it (R07.4.16)",
            )
        } else {
            let Some(row) = REMOVED
                .iter()
                .find(|r| r.name == name && r.method == receiver.is_some())
            else {
                return false;
            };
            let positional: Vec<(&str, bool)> = (args.iter())
                .filter(|a| a.name.is_none())
                .map(|a| {
                    let t = self.text(a.value.span);
                    (t, is_atom(&a.value, t))
                })
                .collect();
            if positional.len() != row.arity {
                return false;
            }
            let named: String = (args.iter())
                .filter_map(|a| {
                    let n = a.name.as_ref()?;
                    Some(format!(
                        ", {}: {}",
                        self.text(n.span),
                        self.text(a.value.span)
                    ))
                })
                .collect();
            let receiver = receiver.map(|r| {
                let t = self.text(r.span);
                (t, is_atom(r, t))
            });
            let ours = fill(row.to, receiver, &positional, &named);
            let edits = vec![(call.span, ours.clone())];
            (ours, edits, row.why)
        };
        self.sink.drop_where(|d| callee_error(d));
        diag.message = format!("`{name}` was removed from std");
        diag.fixes.clear();
        if edits.is_empty() {
            diag.help = Some(why.to_string());
        } else {
            diag.help = Some(format!("{why}: `{ours}`"));
            diag.fixes.push(cpl_diag::Fix {
                title: format!("write `{ours}`"),
                edits,
            });
        }
        self.sink.push(diag);
        true
    }

    /// `a.prefix_excl(op)`: `let pre = a.scan(op)` and `let suf = a.rev.scan(op)` above the
    /// statement, then `[D] i => op(pre[i.l], suf[i.r])`. The scans stay out of the lambda, so
    /// the rewrite stays \(O(n)\). `None` where the statement can't take the lets.
    fn prefix_excl_edits(
        &mut self,
        call: &Expr,
        receiver: Option<&Expr>,
        args: &[Arg],
    ) -> Option<(String, Vec<(Span, String)>)> {
        let (a, op) = match (receiver, args) {
            (Some(a), [op]) => (a, op),
            (None, [a, op]) => (&a.value, op),
            _ => return None,
        };
        if args.iter().any(|x| x.name.is_some()) {
            return None;
        }
        let src = self.src;
        let start = call.span.start as usize;
        let line_start = src[..start].rfind('\n').map_or(0, |i| i + 1);
        let line = &src[line_start..];
        let indent = &line[..line.len() - line.trim_start().len()];
        let stmt = line.trim_start();
        let head = [
            "for ", "while ", "if ", "elif ", "else", "match ", "fn ", "@",
        ];
        let line_end = stmt.find('\n').unwrap_or(stmt.len());
        if head.iter().any(|h| stmt.starts_with(h)) || stmt[..line_end].contains(" for ") {
            return None;
        }
        let op_text = self.text(op.value.span);
        let infix = ["+", "*", "&", "|", "^"].contains(&op_text);
        if !infix && !matches!(op.value.kind, ExprKind::Name(_)) {
            return None;
        }
        let skip = src[..start].matches(PREFIX_EXCL).count();
        let mut lets = String::new();
        let a_text = match &a.kind {
            ExprKind::Name(n) => n.clone(),
            _ => {
                let xs = fresh(src, "xs", skip);
                lets.push_str(&format!("{indent}let {xs} = {}\n", self.text(a.span)));
                xs
            }
        };
        let (pre, suf) = (fresh(src, "pre", skip), fresh(src, "suf", skip));
        lets.push_str(&format!("{indent}let {pre} = {a_text}.scan({op_text})\n"));
        lets.push_str(&format!(
            "{indent}let {suf} = {a_text}.rev.scan({op_text})\n"
        ));
        let both = |x: String, y: String| match infix {
            true => format!("{x} {op_text} {y}"),
            false => format!("{op_text}({x}, {y})"),
        };
        let domain = match self.quietly(|c| c.infer(a)) {
            Ty::Array { index, .. } => match index.as_slice() {
                [Ty::Domain(d) | Ty::Pos(d)]
                    if matches!(self.type_def(d).as_deref(), Some(Def::Domain(_))) =>
                {
                    Some(d.clone())
                }
                _ => None,
            },
            _ => None,
        };
        let ours = match domain {
            Some(d) => format!(
                "[{d}] i => {}",
                both(format!("{pre}[i.l]"), format!("{suf}[i.r]"))
            ),
            None => format!(
                "{a_text}.keys.map(i => {})",
                both(format!("{pre}.cut(i)"), format!("{suf}.cut(i + 1)"))
            ),
        };
        let at = Span::new(line_start as _, line_start as _);
        Some((ours.clone(), vec![(at, lets), (call.span, ours)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str) -> &'static Removed {
        REMOVED.iter().find(|r| r.name == name).unwrap()
    }

    #[test]
    fn rows_fill_their_holes() {
        let x = |t| (t, true);
        let named = ", steps: 60";
        let ok = [x("lo"), x("hi"), x("ok")];
        assert_eq!(
            fill(row("bsearch_real").to, None, &ok, named),
            "(lo..hi).bsearch(ok, steps: 60)"
        );
        assert_eq!(
            fill(row("ternary_max_real").to, None, &ok, ""),
            "(lo..hi).peak(by: ok)"
        );
        assert_eq!(
            fill(row("ternary_min_real").to, None, &ok, named),
            "(lo..hi).valley(by: ok, steps: 60)"
        );
        assert_eq!(
            fill(row("skip_while").to, Some(x("xs")), &[x("p")], ""),
            "xs.drop_while(p)"
        );
        let sum = [x("+"), ("a + b", false)];
        assert_eq!(
            fill(row("suffix_fold").to, None, &sum, ""),
            "(a + b).rev.scan(+)"
        );
    }

    #[test]
    fn fresh_names_skip_used_words() {
        assert_eq!(fresh("let prefix = 1", "pre", 0), "pre");
        assert_eq!(fresh("let pre = 1", "pre", 0), "pre2");
        assert_eq!(fresh("let pre = 1", "pre", 1), "pre3");
    }
}
