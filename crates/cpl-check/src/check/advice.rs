//! The words and fixes of a few errors, kept out of the passes that find them.

use super::*;
use cpl_diag::Fix;
use cpl_syntax::ast::{Arg, BinOp, CmpOp, Expr, ExprKind, FnBody, FnDecl, Name, StmtKind};

/// The fix for a clear operand mistake: a string built with `+`, a value
/// put in a string, or `^` on Bools, which is `!=` (R01.5.9).
pub(super) fn bad_operands(
    diag: &mut Diagnostic,
    (op, op_span): (BinOp, Span),
    (left, right): (&Ty, &Ty),
    into_string: Option<(Span, String)>,
) {
    if let Some((span, text)) = into_string {
        diag.fix(format!("write `{text}`"), span, text);
    } else if *left == Ty::Str || *right == Ty::Str {
        diag.help = Some("put the value in the string: `\"{x}\"`".into());
    } else if op == BinOp::BitXor && *left == Ty::Bool && *right == Ty::Bool {
        diag.fix("two Bools differ by `!=`", op_span, "!=");
    }
}

impl Checker<'_> {
    /// `x.inv` names a method that does work, so it is called with `()`,
    /// as R07.1.7 asks of built-in members.
    pub(super) fn uncalled_method(
        &mut self,
        expr: &Expr,
        base: &Expr,
        name: &Name,
        owner: &str,
        ty: &Ty,
    ) {
        let does_work = match ty {
            Ty::Fn { params, ret } => {
                params.is_empty() && !matches!(**ret, Ty::Unit | Ty::Unknown | Ty::Error)
            }
            // `r.sum` on a type std's `fn sum(self: S, ..)` reaches.
            Ty::Unknown => crate::prelude::work_methods().contains(&name.text),
            _ => false,
        };
        if !does_work {
            return;
        }
        if let Some(Def::Struct { fields, .. }) = self.env.global(owner).as_deref() {
            if fields.iter().any(|f| f.name == name.text) {
                return;
            }
        }
        let after = self.src.get(expr.span.end as usize..).unwrap_or("");
        if after.trim_start().starts_with('(') {
            return;
        }
        let shown = format!("{}.{}", self.quote(base.span), name.text);
        let message = format!("`{shown}` does work, but without `()` it reads like an O(1) field");
        let end = Span::new(expr.span.end, expr.span.end);
        self.sink.push(
            Diagnostic::warning(codes::PARENS_MEAN_WORK, expr.span, message)
                .with_help(format!(
                    "write `{shown}()`: `()` marks a call that does work"
                ))
                .with_fix(format!("write `{shown}()`"), end, "()"),
        );
    }

    /// `a * inv` with no `impl Mul` for `inv`'s type. When `inv` is a
    /// function not yet called (`x.inv`) whose result fits, the fix calls it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn no_operator_impl(
        &mut self,
        op: BinOp,
        op_span: Span,
        rhs: &Expr,
        right: &Ty,
        wants: &[Ty],
        trait_name: impl std::fmt::Display,
        owner: impl std::fmt::Display,
    ) -> Ty {
        let o = op.text();
        if let Ty::Fn { params, ret } = right {
            if params.is_empty() && wants.iter().any(|w| ret.fits(w)) {
                let name = self.quote(rhs.span);
                let inline = matches!(rhs.kind, ExprKind::Field { .. });
                let message = if inline {
                    format!("`{o}` takes a {ret}, but `{name}` is a method that returns one")
                } else {
                    format!("`{o}` takes a {ret}, but `{name}` holds a method, not its result")
                };
                if let Some(diag) = self.error(codes::TYPE_MISMATCH, op_span, message) {
                    if inline {
                        diag.help = Some(format!("call it to get the {ret}: `{name}()`"));
                        let end = Span::new(rhs.span.end, rhs.span.end);
                        diag.fix(format!("write `{name}()`"), end, "()");
                    } else {
                        diag.help =
                            Some(format!("call the method with `()` where `{name}` is made"));
                    }
                }
                return Ty::Error;
            }
        }
        let message =
            format!("`{o}` needs `impl {trait_name}<{right}> for {owner}`, and there is none");
        if let Some(diag) = self.error(codes::MISSING_IMPL, op_span, message) {
            diag.help = Some(format!(
                "write `impl {trait_name}<{right}> for {owner}` to define `{owner} {o} {right}`"
            ));
        }
        Ty::Error
    }

    /// Advice on a `for` header: a counter over a slice, and a pair pattern
    /// over `runs`, whose runs are slices (not `(value, count)` pairs).
    pub(super) fn loop_advice(&mut self, pat: &cpl_syntax::ast::Pattern, iter: &Expr, span: Span) {
        self.counter_over_slice(iter, span);
        let cpl_syntax::ast::PatternKind::Tuple(names) = &pat.kind else {
            return;
        };
        let ExprKind::Call { callee, args } = &iter.kind else {
            return;
        };
        let runs = match &callee.kind {
            ExprKind::Name(n) => n == "runs" && args.len() == 1,
            ExprKind::Field { name, .. } => name.text == "runs" && args.is_empty(),
            _ => false,
        };
        if !runs || names.len() != 2 || args.iter().any(|a| a.name.is_some()) {
            return;
        }
        let message = "each run is a slice, so it can't be taken apart into 2 names".to_string();
        if let Some(diag) = self.error(codes::TYPE_MISMATCH, pat.span, message) {
            let pair = ".map(r => (r.first!, r.len))";
            diag.help = Some(format!(
                "for each value and its count, walk `runs(..){pair}`"
            ));
            let end = Span::new(iter.span.end, iter.span.end);
            diag.fix(format!("write `{pair}`"), end, pair);
        }
    }

    /// E0203 for `name` on a value with these fields and methods: the closest
    /// member, or `len == 0` for an `is_empty` the type spells with `len`.
    pub(super) fn unknown_member(
        &mut self,
        name: &Name,
        base_ty: &Ty,
        field_names: &[String],
        methods: &[(String, Ty)],
    ) {
        let owner = match base_ty {
            Ty::Named { name, .. } | Ty::TypeName(name) => name.clone(),
            _ => String::new(),
        };
        let public =
            |c: &mut Self, m: &String| owner.is_empty() || !c.env.is_private_field(&owner, m);
        let field_names: Vec<String> = field_names
            .iter()
            .filter(|f| public(self, f))
            .cloned()
            .collect();
        let methods: Vec<(String, Ty)> = methods
            .iter()
            .filter(|(m, _)| public(self, m))
            .cloned()
            .collect();
        let habit = (!owner.is_empty())
            .then(|| self.public_foreign_twin(&owner, &name.text))
            .flatten();
        let has =
            |m: &str| methods.iter().any(|(n, _)| n == m) || field_names.iter().any(|f| f == m);
        let empty_by_len = name.text == "is_empty" && has("len");
        let after = self.src.get(name.span.end as usize..).unwrap_or("");
        let range_arg = after
            .strip_prefix('(')
            .and_then(|a| a.split(')').next())
            .is_some_and(|a| a.contains(".."));
        let folds = ["sum", "query", "range_sum", "range_query"].contains(&name.text.as_str());
        let fold_for = folds && range_arg && has("fold");
        let known = field_names
            .iter()
            .chain(methods.iter().map(|(m, _)| m))
            .map(String::as_str)
            .chain(has("range_first").then_some("range"));
        let suggestion = closest(&name.text, known);
        let library_domain = match base_ty {
            Ty::Named { name, .. } | Ty::TypeName(name) => self.has_impl(name, "Index"),
            _ => false,
        };
        let core_only =
            library_domain && super::library_domain::CORE_DOMAIN_ONLY.contains(&name.text.as_str());
        let message = format!("{base_ty} has no field or method `{}`", name.text);
        let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) else {
            return;
        };
        let renamed = library_domain
            .then(|| super::library_domain::index_rename(&name.text))
            .flatten();
        match suggestion {
            _ if renamed.is_some() => {
                let new = renamed.unwrap_or_default();
                diag.help = Some(format!(
                    "a library domain's `{}` is `{new}` since 0.3",
                    name.text
                ));
                diag.fix(format!("write `{new}`"), name.span, new);
            }
            _ if core_only => {
                diag.help = Some(format!(
                    "{base_ty} is a library domain (`impl Index`): it walks in `ord` order, but neighbours, cuts, `at` and `wrap` belong to the ordered core domains; use `x.ord` and `{base_ty}.nth(k)`"
                ));
            }
            _ if habit.is_some() => {
                let own = habit.unwrap_or_default();
                diag.help = Some(format!("`{}` is spelled `{own}` in CP Lang", name.text));
                diag.fix(format!("write `{own}`"), name.span, own);
            }
            _ if empty_by_len => {
                diag.help = Some(format!("a {base_ty} is empty when its `len` is 0"));
                diag.fix("write `len == 0`".to_string(), name.span, "len == 0");
            }
            _ if fold_for => {
                diag.help = Some(format!("a {base_ty} combines a range with `fold`"));
                diag.fix("write `fold`".to_string(), name.span, "fold");
            }
            Some(s) => diag.suggest(name.span, s),
            None if !field_names.is_empty() => {
                let fields = super::expr::list_names(&field_names);
                diag.help = Some(format!("its fields are {fields}"));
            }
            None => {
                let names = self
                    .public_receivers(&owner)
                    .unwrap_or_else(|| methods.iter().map(|(m, _)| m.clone()).collect());
                if let Some(list) = method_list(&names) {
                    if let Some(d) = self.sink.last_mut() {
                        d.help = Some(format!("its methods are {list}"));
                    }
                }
            }
        }
    }

    /// `a `T` is not a `Trait``, then what `T` does have, for an unmet bound.
    pub(super) fn unmet_bound_help(&mut self, ty: &Ty) -> Option<String> {
        let Ty::Named { name, .. } = ty else {
            return None;
        };
        let names = self.public_receivers(name);
        method_list(&names?).map(|list| format!("a `{ty}` offers {list}"))
    }

    /// The public methods a value of struct `owner` calls: `self` methods
    /// and getters, not constructors.
    fn public_receivers(&mut self, owner: &str) -> Option<Vec<String>> {
        let Some(Def::Struct {
            receivers,
            mutators,
            getters,
            ..
        }) = self.type_def(owner).as_deref().cloned()
        else {
            return None;
        };
        let names = receivers.into_iter().chain(mutators).chain(getters);
        Some(
            names
                .filter(|m| !self.env.is_private_field(owner, m))
                .collect(),
        )
    }
}

/// Up to ten public method names, sorted, then how many more: the help for
/// a member or bound a type lacks, so the reader sees what it does have.
fn method_list(names: &[String]) -> Option<String> {
    const SHOWN: usize = 10;
    let mut names: Vec<String> = names
        .iter()
        .filter(|m| !m.starts_with('_'))
        .cloned()
        .collect();
    names.sort();
    names.dedup();
    let more = names.len().saturating_sub(SHOWN);
    names.truncate(SHOWN);
    let list = super::expr::list_names(&names);
    match more {
        _ if names.is_empty() => None,
        0 => Some(list),
        n => Some(format!("{list} and {n} more")),
    }
}

impl Checker<'_> {
    /// The `None` literal; where `none` is bound, the lowercase word is that name.
    pub(super) fn none_literal(&mut self, span: Span) -> Ty {
        let bound = self.local("none").is_some() || self.global("none").is_some();
        if self.text(span) == "none" && bound {
            return self.name_ty("none", span);
        }
        self.old_optional_spelling(span, "None");
        Ty::Unknown
    }

    /// Whether `path` is the `Some(x)` pattern, reporting the old `some` spelling.
    pub(super) fn some_pattern(&mut self, path: &[Name]) -> bool {
        let is_some = path.len() == 1 && path[0].text == "some";
        if is_some {
            self.old_optional_spelling(path[0].span, "Some");
        }
        is_some
    }

    /// `some(x)` and `none`, the spellings before `Some(x)` and `None`
    /// (decision 157): an error whose fix `cpl migrate` applies.
    pub(super) fn old_optional_spelling(&mut self, span: Span, new: &str) {
        let old = self.text(span);
        if old == new || old != new.to_lowercase() || self.local(old).is_some() {
            return;
        }
        // A top-level `let none = ...` is a name; `some` is also the prelude's.
        if old == "none" && self.global(old).is_some() {
            return;
        }
        let message = format!("`{old}` is spelled `{new}` since 0.3");
        if let Some(diag) = self.error(codes::REMOVED_SYNTAX, span, message) {
            diag.help = Some(format!(
                "optionals are `Some(x)` and `None`; `cpl migrate` rewrites every `{old}` in a folder"
            ));
            diag.fix(format!("write `{new}`"), span, new);
        }
    }

    /// `s.reverse()` on a `let`: the reversed view needs no `var`.
    pub(super) fn reverse_view_help(&mut self, name: &str, base: &Expr) {
        let copy = match name {
            "reverse" => format!("read it reversed with `{}.rev`", self.text(base.span)),
            "sort" => format!(
                "take a sorted copy with `{}.sorted()`",
                self.text(base.span)
            ),
            _ => return,
        };
        let codes = [codes::MUT_OF_IMMUTABLE, codes::ASSIGN_TO_LET];
        if let Some(d) = self.sink.last_mut().filter(|d| codes.contains(&d.code)) {
            let help = d.help.take().unwrap_or_default();
            d.help = Some(format!("{help}, or {copy}"));
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
            ("Float" | "Double", [x]) => (
                format!("F64({})", self.text(x.value.span)),
                "a conversion is named by its type",
            ),
            ("to_float" | "toFloat" | "toDouble", []) => match &callee.kind {
                ExprKind::Field { base, .. } if simple(self.text(base.span)) => (
                    format!("F64({})", self.text(base.span)),
                    "a conversion is named by its type",
                ),
                _ => return,
            },
            ("pow", [a, b]) => (
                format!("{} ** {}", operand(self, a), operand(self, b)),
                "power is the operator `**`",
            ),
            ("pow", [a, e, m]) => (
                format!(
                    "pow_mod({}, {}, {})",
                    self.text(a.value.span),
                    self.text(e.value.span),
                    self.text(m.value.span)
                ),
                "a power modulo `m` is `pow_mod`",
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
        if let (Ty::Str, Ty::Char, Some(c)) = (got, want, crate::char_literal(self.text(at.span))) {
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
        if let Some(new) = super::library_domain::index_rename(&name.text) {
            if self.index_like(base_ty, &name.text) {
                // `from_idx` gave an optional; `try_at` keeps that, `nth` traps instead.
                let fix = if new == "nth" { "try_at" } else { new };
                let message = format!("`{}` was renamed to `{new}` in 0.3", name.text);
                if let Some(diag) = self.error(codes::REMOVED_SYNTAX, name.span, message) {
                    diag.help = Some("a domain reads `D.count`, `x.ord` and `D.nth(k)`, which traps outside `0..D.count`; `D.try_at(k)` gives an optional (R04.2.26)".to_string());
                    diag.fix(format!("write `{fix}`"), name.span, fix);
                }
                return true;
            }
        }
        if name.text == "bounds" && matches!(base_ty, Ty::Domain(_)) {
            let message = "`D.bounds` was renamed to `D.cuts`".to_string();
            self.error(codes::REMOVED_SYNTAX, name.span, message);
            return true;
        }
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
        fn literal(e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) => true,
                ExprKind::Unary { expr, .. } => literal(expr),
                ExprKind::Binary { lhs, rhs, .. } => literal(lhs) && literal(rhs),
                ExprKind::Paren(inner) => literal(inner),
                _ => false,
            }
        }
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
        let Some(text) = literal(rhs) else {
            // `s + 1`, `n + s`: both sides placed, when each is a name,
            // field or number.
            let simple = |e: &Expr| {
                matches!(
                    e.kind,
                    ExprKind::Name(_) | ExprKind::Int(_) | ExprKind::Field { .. }
                )
            };
            let (l, r) = (self.quote(lhs.span), self.quote(rhs.span));
            let placed = simple(lhs) && simple(rhs) && !l.contains('\n') && !r.contains('\n');
            return placed.then(|| (span, format!("\"{{{l}}}{{{r}}}\"")));
        };
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
        "null" | "nil" | "none" | "NULL" | "nullptr" => Some("None"),
        _ => None,
    }
}

/// Python's conversion builtins and `math.`, as CP Lang writes them.
/// `xs.shuffled()` where std has the in-place `shuffle`: name the copy-then-change form.
pub(super) fn in_place_twin(name: &str, recv: &str) -> Option<String> {
    let stems = [name.strip_suffix('d'), name.strip_suffix("ed")];
    let stems = stems.into_iter().flatten().filter(|s| !s.is_empty());
    // `[1..=n]` is a list of one range; its numbers are `List(1..=n)` (R07.8.1).
    let inner = recv.strip_prefix('[').and_then(|r| r.strip_suffix(']'));
    let (copy, lead) = match inner
        .filter(|r| r.contains("..") && !r.contains([',', '[']) && !r.contains(" for "))
    {
        Some(range) => (
            format!("List({range})"),
            format!("`{recv}` is a list holding one range; the numbers are `List({range})`. "),
        ),
        None => (format!("{recv}.clone()"), String::new()),
    };
    for stem in stems {
        // A list method `builtin_catalog` runs, so `cpl run` has it.
        let listed = crate::builtins::BUILTIN_MEMBERS
            .iter()
            .any(|m| m.receiver == crate::builtins::Receiver::List && m.mutates && m.name == stem);
        if listed {
            return Some(format!(
                "{lead}`{stem}` changes the list in place and returns nothing; for a changed copy, write `var ys = {copy}`, then `ys.{stem}()`"
            ));
        }
        if let Some((owner, module)) = std_in_place_method(stem) {
            let var = owner.to_lowercase();
            return Some(format!(
                "{lead}`{stem}` is a method of `{owner}` (`std.{module}`) that changes a list in place; for a changed copy, write `var ys = {copy}`, then `{var}.{stem}(mut ys)` with a `var {var}: {owner}`"
            ));
        }
    }
    None
}

/// A std method `fn stem<S: IndexedMut>(mut self, mut a: S)` that changes the sequence it is
/// given, as `Rng.shuffle`: its type's name and module, read from the std source.
fn std_in_place_method(stem: &str) -> Option<(String, String)> {
    for (module, text) in crate::modules::std_modules() {
        let mut owner = None;
        for line in text.lines() {
            if let Some(rest) = line
                .strip_prefix("struct ")
                .or_else(|| line.strip_prefix("pub struct "))
            {
                owner = Some(
                    rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                        .next()?
                        .to_string(),
                );
            } else if !line.starts_with([' ', '\t'])
                && !line.trim().is_empty()
                && !line.starts_with("//")
                && !line.starts_with('@')
            {
                owner = None;
            }
            let Some(head) = line.trim_start().strip_prefix("pub fn ") else {
                continue;
            };
            let after = head
                .strip_prefix(stem)
                .filter(|r| r.starts_with(['<', '(']));
            if let (Some(after), Some(owner)) = (after, &owner) {
                if after.contains("(mut self, mut ") && after.contains("IndexedMut") {
                    return Some((owner.clone(), module.replace('/', ".")));
                }
            }
        }
    }
    None
}

pub(super) fn foreign_call(name: &str) -> Option<&'static str> {
    match name {
        "str" | "to_string" | "toString" => Some("make a string by interpolation: `\"{x}\"`"),
        "int" | "stoi" | "atoi" | "parseInt" => {
            Some("read a number from a string with `s.parse<Int>()!`")
        }
        "float" | "stod" | "parseFloat" => {
            Some("read a number from a string with `s.parse<F64>()!`")
        }
        "input" | "cin" | "scanf" => Some("read the input with `read`: `let n: Int = read()`"),
        "cout" | "printf" | "puts" => Some("write the output with `print`: `print(a, b)`"),
        "endl" => Some("`print` ends the line itself: `print(a, b)` writes `a b` and a newline"),
        "ord" => Some("a character's code is `Int(c)`"),
        "chr" => Some("the character with code `i` is `Char(i)`"),
        "dict" => Some("an empty map is `Map<K, V>()`, or `[:]` where its type is known"),
        "math" | "Math" => Some("math functions are methods here: `x.sqrt`, `x.abs`, `x.isqrt`"),
        _ => None,
    }
}

/// C++'s streams: `cin >> n` is `n = read()`, and `, endl` goes, since
/// `print` ends the line.
pub(super) fn stream_fix(src: &str, name: &str, span: Span) -> Option<(Span, String)> {
    let rest = src.get(span.end as usize..)?.split('\n').next()?.trim_end();
    match name {
        "cin" => {
            let v = rest.strip_prefix(" >> ")?;
            let ident = v.chars().all(|c| c.is_alphanumeric() || c == '_');
            (ident && !v.is_empty()).then(|| {
                let end = span.end + rest.len() as u32;
                (Span::new(span.start, end), format!("{v} = read()"))
            })
        }
        "endl" => src
            .get(..span.start as usize)?
            .ends_with(", ")
            .then(|| (Span::new(span.start - 2, span.end), String::new())),
        _ => None,
    }
}

/// `a.begin()`, C++'s iterators: a call takes the whole sequence.
pub(super) fn foreign_method(name: &str) -> Option<&'static str> {
    matches!(name, "begin" | "end" | "rbegin" | "rend" | "cbegin" | "cend").then_some(
        "CP Lang has no iterators: a call takes the whole sequence, so `sort(a.begin(), a.end())` is `a.sort()`",
    )
}

/// `sort(a.begin(), a.end())` as `a.sort()`.
pub(super) fn iterator_pair_fix(src: &str, base: Span, name: Span) -> Option<Fix> {
    let x = src.get(base.start as usize..base.end as usize)?;
    let tail = format!("(), {x}.end())");
    if !src.get(name.end as usize..)?.starts_with(&tail) {
        return None;
    }
    let head = src.get(..base.start as usize)?;
    let f = ["sort", "reverse"]
        .into_iter()
        .find(|f| head.ends_with(&format!("{f}(")))?;
    let start = base.start - f.len() as u32 - 1;
    let new = format!("{x}.{f}()");
    Some(Fix {
        title: format!("write `{new}`"),
        edits: vec![(Span::new(start, name.end + tail.len() as u32), new)],
    })
}

/// `xs.max_by(f)`, the C++/Rust spelling: options are named arguments,
/// so it is `xs.max(by: f)`. Only for a base std declares with `by:`.
pub(super) fn by_twin(src: &str, name: &str, span: Span) -> Option<(String, Fix)> {
    let base = by_base(name)?;
    let open = span.end as usize;
    let mut edits = vec![(span, base.to_string())];
    if src.get(open..open + 1) == Some("(") && src.get(open + 1..open + 2) != Some(")") {
        edits.push((Span::new(span.end + 1, span.end + 1), "by: ".into()));
    } else {
        return None;
    }
    Some((
        format!("write `.{base}(by: ..)`: options are named arguments, so there is one `{base}` and no `_by` twin"),
        Fix {
            title: format!("write `{base}(by: ..)`"),
            edits,
        },
    ))
}

/// The one std name a `_by` twin (`max_by`, `sort_by_key`) folds into.
fn by_base(name: &str) -> Option<&'static str> {
    let base = name.strip_suffix("_by").or(name.strip_suffix("_by_key"))?;
    Some(match base {
        "max" => "max",
        "min" => "min",
        "sort" => "sort",
        "sorted" => "sorted",
        "argmax" | "position_max" => "argmax",
        "argmin" | "position_min" => "argmin",
        "is_sorted" => "is_sorted",
        "runs" => "runs",
        "group" => "group",
        _ => return None,
    })
}

/// `min_by(xs, f)`, a `_by` twin called as a free function: `xs.min(by: f)`.
pub(crate) fn free_by_twin(src: &str, name: &str, span: Span) -> Option<(String, Fix)> {
    let base = by_base(name)?;
    let rest = src.get(span.end as usize..)?.strip_prefix('(')?;
    let mut depth = 0;
    let mut comma = None;
    for (i, c) in rest.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => break,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                comma = Some(i);
                break;
            }
            '"' | '\'' => return None,
            _ => {}
        }
    }
    let comma = comma?;
    let recv = rest[..comma].trim();
    let recv = recv.strip_prefix("mut ").unwrap_or(recv);
    if recv.is_empty() {
        return None;
    }
    let recv = if recv.contains(' ') || recv.contains("..") {
        format!("({recv})")
    } else {
        recv.to_string()
    };
    let args = &rest[comma + 1..];
    let end = span.end + 1 + (comma + 1 + args.len() - args.trim_start().len()) as u32;
    let shown = format!("{recv}.{base}(by: ..)");
    Some((
        format!("write `{shown}`: options are named arguments, so there is one `{base}` and no `_by` twin"),
        Fix {
            title: format!("write `{shown}`"),
            edits: vec![(Span::new(span.start, end), format!("{recv}.{base}(by: "))],
        },
    ))
}

/// The E0208 fix for an uppercase top-level value: rename it and every use
/// in the file's items, when its lowercase name is not a word anywhere in it.
pub(crate) fn rename_values(
    mut diags: Vec<cpl_diag::Diagnostic>,
    ranges: &[cpl_syntax::ItemRange],
    checks: &[std::sync::Arc<crate::db::ItemCheck>],
    src: &str,
) -> Vec<cpl_diag::Diagnostic> {
    let at = |s: Span| src.get(s.start as usize..s.end as usize).unwrap_or("");
    let words: std::collections::HashSet<&str> = src
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .collect();
    let refs: Vec<(Span, &str)> = ranges
        .iter()
        .zip(checks)
        .flat_map(|(r, c)| {
            c.refs
                .iter()
                .map(move |g| (g.span.shift(r.span.start), &*g.name))
        })
        .collect();
    for d in diags
        .iter_mut()
        .filter(|d| d.code == codes::VALUE_NAME_CASE)
    {
        let old = at(d.span);
        let line = src[..d.span.start as usize]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let top_level = ["let ", "var "].iter().any(|k| src[line..].starts_with(k));
        let lower = old.to_lowercase();
        let taken = words.contains(lower.as_str()) || !d.fixes.is_empty();
        if !top_level || old.is_empty() || taken {
            continue;
        }
        let uses = refs
            .iter()
            .filter(|(s, name)| *name == old && at(*s) == old);
        let mut edits: Vec<_> = std::iter::once(d.span)
            .chain(uses.map(|(s, _)| *s))
            .map(|s| (s, lower.clone()))
            .collect();
        edits.sort_by_key(|(s, _)| s.start);
        edits.dedup_by_key(|(s, _)| s.start);
        let title = format!("rename `{old}` to `{lower}` everywhere");
        d.fixes.push(Fix { title, edits });
    }
    diags
}

/// E0802: a bare `with` covers the rest of its block, so it opens it.
pub(crate) const WITH_FIRST_HELP: &str = "a bare `with` covers the rest of its block: move it to the block's first line, or end it with `:` and indent the lines it covers under it";

/// L0601: `xs[0] = xs.pop()!` changes `xs` on the right; `parts` compares how deep the
/// assigned place goes against the changed one.
pub(crate) fn same_place(
    place: &Expr,
    target: &Expr,
    value: &Expr,
    op: cpl_syntax::ast::AssignOp,
    parts: std::cmp::Ordering,
    why: String,
) -> (String, String) {
    use super::lends::show;
    let (changed, assigned) = (show(place), show(target));
    let why = match parts {
        std::cmp::Ordering::Greater => format!("`{assigned}` is part of `{changed}`"),
        std::cmp::Ordering::Less => format!("`{changed}` is part of `{assigned}`"),
        std::cmp::Ordering::Equal => why,
    };
    let message = format!(
        "`{changed}` is changed on the right side of an assignment to `{assigned}`, and {why}"
    );
    let lends_mut =
        matches!(&value.kind, ExprKind::Call { args, .. } if args.iter().any(|a| a.mutable));
    let first = match op {
        cpl_syntax::ast::AssignOp::Set if !lends_mut => {
            format!("first, `let v = {}`, then `{assigned} = v`", show(value))
        }
        _ => "into a `let` first, then assign".into(),
    };
    let help = format!(
        "compute the right side {first}: otherwise the order the two sides run in decides the result"
    );
    (message, help)
}

impl Checker<'_> {
    /// A core domain, a position, an enum index, `Int`, `Char` or a type parameter: the
    /// types whose pre-0.3 `K`, `to_idx` and `from_idx` are now `count`, `ord` and `nth`.
    fn index_like(&mut self, ty: &Ty, member: &str) -> bool {
        match ty {
            Ty::Int | Ty::Char | Ty::Pos(_) | Ty::Domain(_) => true,
            Ty::Param(p) => !self
                .row_bounds
                .get(p)
                .is_some_and(|row| row.iter().any(|(field, _)| field == member)),
            Ty::TypeName(n) | Ty::Named { name: n, .. } => {
                n == "Char"
                    || self.finite_index_enum_size(n).is_some()
                    || self.param_bounds.iter().any(|(p, _)| p == n)
            }
            _ => false,
        }
    }
}

/// "did you mean `s`?" for a member, naming the module and adding the
/// `use` when `.s` on this receiver needs one (errors/R7).
pub(crate) fn suggest_member(
    diag: &mut Diagnostic,
    src: &str,
    span: Span,
    s: String,
    on_str: bool,
) {
    let auto = crate::prelude::methods().iter().any(|(_, m)| m.text == s);
    let Some(m) = crate::prelude::method_module(&s, on_str).filter(|_| !auto) else {
        return diag.suggest(span, s);
    };
    let (at, import) = crate::prelude::import_edit(src, &m, &s);
    diag.help = Some(format!("did you mean `{s}`, from `std.{m}`?"));
    diag.fixes.push(Fix {
        title: format!("change to `{s}` and add it to `use std.{m}`"),
        edits: vec![(at, import), (span, s)],
    });
}

impl<'a> Checker<'a> {
    /// `Set` or `Map` when `expr` is a `.range(..)` view of one, through
    /// parens and `.rev`: a view whose length isn't counted.
    pub(super) fn uncounted_sorted_range_owner(&mut self, expr: &Expr) -> Option<&'static str> {
        let mut cur = expr;
        loop {
            match &cur.kind {
                ExprKind::Paren(inner) => cur = inner,
                ExprKind::Field { base, name } if name.text == "rev" => cur = base,
                ExprKind::Call { callee, .. } => {
                    let ExprKind::Field { base, name } = &callee.kind else {
                        return None;
                    };
                    if name.text != "range" {
                        return None;
                    }
                    let base_ty = self.quietly(|c| c.infer(base));
                    return match base_ty {
                        Ty::Named { name, .. } if name == "Set" => Some("Set"),
                        Ty::Named { name, .. } if name == "Map" => Some("Map"),
                        _ => None,
                    };
                }
                _ => return None,
            }
        }
    }
}
