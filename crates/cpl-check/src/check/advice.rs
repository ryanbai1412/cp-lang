//! The words and fixes of a few errors, kept out of the passes that find them.

use super::*;
use cpl_diag::Fix;
use cpl_syntax::ast::{Arg, CmpOp, Expr, ExprKind, FnBody, FnDecl, Name, StmtKind};

impl Checker<'_> {
    /// `s.reverse()` on a `let`: the reversed view needs no `var`.
    pub(super) fn reverse_view_help(&mut self, name: &str, base: &Expr) {
        let rev = format!("{}.rev", self.text(base.span));
        let codes = [codes::MUT_OF_IMMUTABLE, codes::ASSIGN_TO_LET];
        let last = self.sink.last_mut();
        if let Some(d) = last.filter(|d| name == "reverse" && codes.contains(&d.code)) {
            let help = d.help.take().unwrap_or_default();
            d.help = Some(format!("{help}, or read it reversed with `{rev}`"));
        }
    }

    /// `enumerate(xs)`, `reversed(xs)`, `pow(a, b)`, `s.charAt(i)`: foreign
    /// functions for what CP Lang spells as a member or an operator, so the
    /// unresolved name gets `xs.items` or `a ** b`.
    pub(super) fn member_for_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) {
        let (name, at, code) = match &callee.kind {
            ExprKind::Name(name) => (name, callee.span, codes::UNRESOLVED_NAME),
            ExprKind::Field { name, .. } => (&name.text, name.span, codes::UNKNOWN_FIELD),
            _ => return,
        };
        let operand = |c: &Self, a: &Arg| {
            let t = c.text(a.value.span);
            let atom = !t.contains(' ') || (t.starts_with('[') && t.ends_with(']'));
            if atom {
                t.to_string()
            } else {
                format!("({t})")
            }
        };
        let simple = |t: &str| {
            t.chars()
                .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.')
        };
        let input = args.first().and_then(|x| match &x.value.kind {
            ExprKind::Call { callee, args } if args.is_empty() => {
                matches!(&callee.kind, ExprKind::Name(n) if n == "input").then_some(callee.span)
            }
            _ => None,
        });
        let read = match name.as_str() {
            "int" => Some("Int"),
            "float" => Some("F64"),
            _ => None,
        };
        if let (Some(at), Some(_), [_]) = (input, read, args) {
            self.sink
                .drop_where(|d| d.code == codes::UNRESOLVED_NAME && d.span == at);
        }
        let (ours, what) = match (name.as_str(), args) {
            ("int" | "float", [_]) if input.is_some() => (
                format!("read<{}>()", read.unwrap_or("Int")),
                "a value is read from the input with `read`",
            ),
            ("enumerate", [xs]) => (
                format!("{}.items", operand(self, xs)),
                "index-element pairs are a member",
            ),
            ("reversed", [xs]) => (
                format!("{}.rev", operand(self, xs)),
                "the reversed view is a member",
            ),
            ("range", [_] | [_, _]) => {
                let bound = |c: &Self, a: &Arg| {
                    let t = c.text(a.value.span);
                    let plain = t
                        .chars()
                        .all(|ch| ch.is_alphanumeric() || " _.+-*/%()[]".contains(ch));
                    if plain {
                        t.to_string()
                    } else {
                        format!("({t})")
                    }
                };
                let (lo, hi) = match args {
                    [n] => ("0".to_string(), bound(self, n)),
                    [a, b] => (bound(self, a), bound(self, b)),
                    _ => return,
                };
                let next = self.text(Span::new(call.span.end, call.span.end + 1));
                let range = if next == "." {
                    format!("({lo}..{hi})")
                } else {
                    format!("{lo}..{hi}")
                };
                (range, "a range is written")
            }
            ("str", [x]) if simple(self.text(x.value.span)) => (
                format!("\"{{{}}}\"", self.text(x.value.span)),
                "a string is made by interpolation",
            ),
            ("to_string" | "toString", []) => match &callee.kind {
                ExprKind::Field { base, .. } if simple(self.text(base.span)) => (
                    format!("\"{{{}}}\"", self.text(base.span)),
                    "a string is made by interpolation",
                ),
                _ => return,
            },
            ("pow", [a, b]) => (
                format!("{} ** {}", operand(self, a), operand(self, b)),
                "power is the operator `**`",
            ),
            ("charAt", [i]) => match &callee.kind {
                ExprKind::Field { base, .. } => (
                    format!("{}[{}]", self.text(base.span), self.text(i.value.span)),
                    "a character is an index",
                ),
                _ => return,
            },
            ("substr" | "substring", [a, b]) => match &callee.kind {
                ExprKind::Field { base, .. } => {
                    let (a, b) = (self.text(a.value.span), self.text(b.value.span));
                    let end = if name == "substr" {
                        format!("{a} + {b}")
                    } else {
                        b.to_string()
                    };
                    (
                        format!("{}[{a}..{end}]", self.text(base.span)),
                        "a substring is a slice",
                    )
                }
                _ => return,
            },
            _ => return,
        };
        let Some(diag) = self
            .sink
            .last_mut()
            .filter(|d| d.code == code && d.span == at && args.iter().all(|a| a.name.is_none()))
        else {
            return;
        };
        diag.help = Some(format!("{what} in CP Lang: `{ours}`"));
        diag.fixes.clear();
        diag.fix(format!("write `{ours}`"), call.span, ours);
    }

    /// `"a"` where a `Char` is wanted: a character is in single quotes.
    pub(super) fn char_literal_fix(&mut self, got: &Ty, want: &Ty, at: &Expr) -> bool {
        if let (Ty::Str, Ty::Char, Some(c)) = (got, want, crate::char_literal(&self.text(at.span)))
        {
            if let Some(diag) = self.sink.last_mut().filter(|d| d.span == at.span) {
                diag.help = Some(format!("a character is written in single quotes: `{c}`"));
                diag.fix(format!("write `{c}`"), at.span, c);
            }
        }
        false
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

    /// `s.chars()`, `xs.iter()`: a call CP Lang leaves out, so its fix drops it.
    pub(super) fn iter_call_help(&mut self, base: &Expr, base_ty: &Ty, name: &Name) {
        let call = Span::new(base.span.end, name.span.end + 2);
        let iter_call = crate::builtins::foreign_iter_call(&name.text)
            && matches!(base_ty, Ty::Str | Ty::Named { .. })
            && self.text(Span::new(name.span.end, call.end)) == "()";
        let last = self
            .sink
            .last_mut()
            .filter(|d| iter_call && d.code == codes::UNKNOWN_FIELD && d.span == name.span);
        if let Some(diag) = last {
            diag.help = Some("a list or string is iterated as it is: `for x in xs`".into());
            diag.fixes.clear();
            diag.fix(format!("remove `.{}()`", name.text), call, "");
        }
    }

    /// `.len` on a lazy sequence, which has no stored length.
    pub(super) fn lazy_len(&mut self, base_ty: &Ty, name: &Name) {
        let message = format!("`.len` requires `Sized`, which `{base_ty}` does not implement");
        if let Some(diag) = self.error(codes::MISSING_IMPL, name.span, message) {
            diag.help =
                Some("a lazy sequence has no stored length: `.count()` walks it once".into());
            diag.fix("write `.count()`", name.span, "count()");
        }
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

    /// A block fn with no `-> T` and no value on its last line gives
    /// nothing (R06.1.1), so a `return` with a value there needs `-> T`.
    pub(super) fn note_ret_slot(&mut self, f: &FnDecl) {
        let FnBody::Block(block) = &f.body else {
            return;
        };
        let tail_value = matches!(block.stmts.last().map(|s| &s.kind), Some(StmtKind::Expr(_)));
        if f.ret.is_none() && !tail_value {
            let slot = f.uses.is_empty().then(|| Span::point(f.sig_span.end));
            self.ret_slots.insert(f.name.text.clone(), slot);
        }
    }

    /// `return x` in a fn with no `-> T`, which gives nothing. Reported
    /// once per fn: every other `return` has the same cause.
    pub(super) fn undeclared_return(&mut self, value: &Expr, ty: &Ty) {
        let Some((name, _, false, _)) = self.fns.last().cloned() else {
            return;
        };
        if name.is_empty() || matches!(ty, Ty::Unit | Ty::Unknown | Ty::Error) {
            return;
        }
        let Some(slot) = self.ret_slots.remove(&name) else {
            return;
        };
        let message = format!("`{name}` returns a value, but its signature has no return type");
        if let Some(diag) = self.error(codes::TYPE_MISMATCH, value.span, message) {
            diag.help = Some(format!("a fn without `-> T` gives nothing: add `-> {ty}`"));
            if let Some(slot) = slot {
                diag.fix(format!("add `-> {ty}`"), slot, format!(" -> {ty}"));
            }
        }
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
        "input" => Some("read the input with `read`: `let n: Int = read()`"),
        "ord" => Some("a character's code is `Int(c)`"),
        "chr" => Some("the character with code `i` is `Char(i)`"),
        "dict" => Some("an empty map is `Map<K, V>()`, or `[:]` where its type is known"),
        "math" | "Math" => Some("math functions are methods here: `x.sqrt`, `x.abs`, `x.isqrt`"),
        _ => None,
    }
}
