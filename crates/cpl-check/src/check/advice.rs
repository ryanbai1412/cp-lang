//! The words and fixes of a few errors, kept out of the passes that find them.

use super::*;
use cpl_diag::Fix;
use cpl_syntax::ast::{Arg, CmpOp, Expr, ExprKind};

impl Checker<'_> {
    /// `enumerate(xs)`, `reversed(xs)`: Python's free functions for what
    /// CP Lang reads as a member, so the unresolved name gets `xs.items`.
    pub(super) fn member_for_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) {
        let (ExprKind::Name(name), [arg]) = (&callee.kind, args) else {
            return;
        };
        let (member, what) = match name.as_str() {
            "enumerate" => ("items", "index-element pairs are"),
            "reversed" => ("rev", "the reversed view is"),
            _ => return,
        };
        let xs = self.text(arg.value.span);
        let xs = if xs.contains(' ') && !(xs.starts_with('[') && xs.ends_with(']')) {
            format!("({xs})")
        } else {
            xs.to_string()
        };
        let Some(diag) = self.sink.last_mut().filter(|d| {
            d.code == codes::UNRESOLVED_NAME && d.span == callee.span && arg.name.is_none()
        }) else {
            return;
        };
        let ours = format!("{xs}.{member}");
        diag.help = Some(format!("{what} a member in CP Lang: `{ours}`"));
        diag.fixes.clear();
        diag.fix(format!("write `{ours}`"), call.span, ours);
    }

    /// `x in xs` on a list: `in` is sub-linear only, so a scan is spelled
    /// out as `xs.contains(x)`.
    pub(super) fn list_in(&mut self, op: CmpOp, x: &Expr, xs: &Expr, ty: &Ty, alone: bool) {
        let message = format!("`in` requires `Contains`, which `{ty}` does not implement");
        let (l, r) = (self.text(x.span), self.text(xs.span));
        let bracketed = r.starts_with('[') && r.ends_with(']');
        let r = if r.contains(' ') && !bracketed {
            format!("({r})")
        } else {
            r.to_string()
        };
        let not = if matches!(op, CmpOp::NotIn) {
            "not "
        } else {
            ""
        };
        let scan = format!("{not}{r}.contains({l})");
        let whole = Span::new(x.span.start, xs.span.end);
        if let Some(diag) = self.error(codes::MISSING_IMPL, xs.span, message) {
            diag.help = Some(
                "`in` is for sets, maps and ranges, where it is fast; a list is scanned in O(n) with `xs.contains(x)`".into(),
            );
            if alone {
                diag.fix(format!("scan the list: `{scan}`"), whole, scan);
            }
        }
    }

    /// `var m: Map<K, V> = {}`: braces make a block, so the empty container
    /// is reported as one, with its spelling as the fix.
    pub(super) fn empty_braces(&mut self, declared: &Ty, init: &Expr) {
        let empty = match declared {
            Ty::Named { name, .. } if name == "Map" => "[:]",
            Ty::Named { name, .. } if name == "Set" => "Set()",
            Ty::Named { name, .. } if name == "List" => "[]",
            _ => return,
        };
        let braces = self.text(init.span).split_whitespace().collect::<String>() == "{}";
        let Some(diag) = self
            .sink
            .last_mut()
            .filter(|d| braces && d.span == init.span)
        else {
            return;
        };
        diag.message = format!("`{{}}` is an empty block in CP Lang, not an empty {declared}");
        diag.help = Some(format!("an empty {declared} is `{empty}`"));
        diag.fix(format!("write `{empty}`"), init.span, empty);
    }

    /// A member that another version or language has and CP Lang spells
    /// differently: reported, with the fix where there is one.
    pub(super) fn member_advice(&mut self, base_ty: &Ty, name: &cpl_syntax::ast::Name) -> bool {
        if name.text == "bsearch_first" {
            let message =
                "`.bsearch_first(...)` was removed; use `.first_true(...)` or `.bsearch(...)`";
            self.error(codes::REMOVED_SYNTAX, name.span, message.to_string());
            return true;
        }
        // `p.first`, `p.second`: C++'s pair, where a tuple numbers its parts.
        let Ty::Tuple(parts) = base_ty else {
            return false;
        };
        let part = ["first", "second", "third"]
            .iter()
            .position(|p| *p == name.text);
        let Some(i) = part.filter(|&i| i < parts.len()) else {
            return false;
        };
        let last = parts.len() - 1;
        let message = format!("a tuple has no `.{}`: its parts have numbers", name.text);
        if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
            diag.help = Some(format!("the parts of this tuple are `.0` to `.{last}`"));
            diag.fix(format!("change to `.{i}`"), name.span, i.to_string());
        }
        true
    }

    /// `let N = 5` as `const N = 5`: only a literal can be a constant as is.
    pub(super) fn const_fix(&self, keyword: &str, name: Span, init: Option<&Expr>) -> Option<Fix> {
        let literal = |e: &Expr| {
            matches!(
                e.kind,
                ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_)
            )
        };
        let at = Span::new(name.start.checked_sub(4)?, name.start - 1);
        (keyword == "let" && init.is_some_and(literal) && self.text(at) == "let").then(|| Fix {
            title: "make it a `const`".into(),
            edits: vec![(at, "const".into())],
        })
    }

    /// `"a" + n` as `"a{n}"`, and `n + "a"` as `"{n}a"`.
    pub(super) fn string_fix(&self, lhs: &Expr, rhs: &Expr) -> Option<(Span, String)> {
        let literal = |e: &Expr| {
            let text = self.quote(e.span);
            let plain = matches!(e.kind, ExprKind::Str(_))
                && text.len() >= 2
                && text.starts_with('"')
                && text.ends_with('"')
                && !text.starts_with("\"\"\"");
            plain.then_some(text)
        };
        let span = Span::new(lhs.span.start, rhs.span.end);
        if let Some(text) = literal(lhs) {
            let value = self.quote(rhs.span);
            return Some((span, format!("{}{{{value}}}\"", &text[..text.len() - 1])));
        }
        let text = literal(rhs)?;
        Some((
            span,
            format!("\"{{{}}}{}", self.quote(lhs.span), &text[1..]),
        ))
    }
}

/// E0601's message and help: one place passed twice is said plainly.
pub(super) fn lent_twice(a: &Arg, b: &Arg, why: &str) -> (String, String) {
    let (first, second) = (super::lends::show(&a.value), super::lends::show(&b.value));
    if first == second {
        let how = if a.mutable && b.mutable {
            "both times `mut`"
        } else {
            "once `mut`"
        };
        let reason = if why.contains("only when") {
            why
        } else {
            "a call can't change one place through two names"
        };
        return (
            format!("`{first}` is lent to this call twice, {how}: {reason}"),
            format!("pass `{first}` once"),
        );
    }
    let how = if a.mutable && b.mutable {
        "both are"
    } else {
        "one is"
    };
    (
        format!("`{first}` and `{second}` may be the same storage, and {how} lent `mut` to this call: {why}"),
        "guard it (`if i != j: ...`), state it (`assert i != j`), or copy the read-only one first".into(),
    )
}

/// E0407: point at the first variant with the tag, and say what to change.
pub(super) fn duplicate_tag(diag: &mut Diagnostic, first: Span) {
    diag.labels.push(cpl_diag::Label {
        span: first,
        message: "first `@tag` here".to_string(),
    });
    diag.help = Some("input names one variant per tag: give one of them another `@tag`".into());
}

/// `True` or `null` from another language, as CP Lang spells the value.
pub(super) fn foreign_value(name: &str) -> Option<&'static str> {
    match name {
        "True" | "TRUE" => Some("true"),
        "False" | "FALSE" => Some("false"),
        "null" | "nil" | "None" | "NULL" | "nullptr" => Some("none"),
        _ => None,
    }
}

/// Python's conversion builtins and `math.`, as CP Lang writes them.
pub(super) fn foreign_call(name: &str) -> Option<&'static str> {
    match name {
        "str" | "to_string" | "toString" => Some("make a string by interpolation: `\"{x}\"`"),
        "int" | "stoi" | "atoi" | "parseInt" => {
            Some("read a number from a string with `s.parse<Int>()!`")
        }
        "float" | "stod" | "parseFloat" => {
            Some("read a number from a string with `s.parse<F64>()!`")
        }
        "math" | "Math" => Some("math functions are methods here: `x.sqrt`, `x.abs`, `x.isqrt`"),
        _ => None,
    }
}
