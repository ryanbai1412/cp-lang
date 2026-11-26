//! The words and fixes of a few errors, kept out of the passes that find them.

pub(crate) use super::foreign::*;
use super::*;
use cpl_diag::habits;
use cpl_diag::msgs;
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
        diag.fix(habits::WRITE.say(&[&text]), span, text);
    } else if *left == Ty::Str || *right == Ty::Str {
        diag.help = Some(habits::BAD_OPERANDS_2.say(&[]));
    } else if op == BinOp::BitXor && *left == Ty::Bool && *right == Ty::Bool {
        diag.fix(habits::BAD_OPERANDS_3.say(&[]), op_span, "!=");
    }
}

/// Whether `text` is one parenthesized expression, `(a + b)`, not `(a) + (b)`.
pub(super) fn wrapped(text: &str) -> bool {
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        depth += i32::from(c == '(') - i32::from(c == ')');
        if depth == 0 {
            return c == ')' && i == text.len() - 1;
        }
    }
    false
}

impl Checker<'_> {
    /// `x.inv` names a method that does work, so it is called with `()`,
    /// as R07.1.7 asks of built-in members.
    /// Whether `x.name` on an `owner` names an `@get`, as member resolution
    /// reads it: `Rebased(a).items` is std's `@get items(self: S)` view, not
    /// the `items` its `impl Keyed` writes (R02.6.6).
    fn reads_as_getter(&mut self, name: &str, owner: &str) -> bool {
        self.method_candidates(name).into_iter().any(|(_, def)| {
            matches!(&*def, Def::Fn { getter: true, ty: Ty::Fn { params, .. }, .. }
            if params.first().is_some_and(|p| match &p.ty {
                Ty::Param(_) => true,
                Ty::Named { name, .. } => name == owner,
                _ => false,
            }))
        })
    }

    /// `()` follows `expr`, or `expr` is the callee of the call being checked.
    pub(super) fn is_called(&self, expr: &Expr) -> bool {
        self.callee == Some(expr.span)
            || called_next(self.src.get(expr.span.end as usize..).unwrap_or(""))
    }

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
            Ty::Unknown => crate::prelude::is_work_method(&name.text),
            _ => false,
        };
        if !does_work || self.reads_as_getter(&name.text, owner) {
            return;
        }
        if let Some(Def::Struct { fields, .. }) = self.env.global(owner).as_deref() {
            if fields.iter().any(|f| f.name == name.text) {
                return;
            }
        }
        if self.is_called(expr) {
            return;
        }
        let shown = format!("{}.{}", self.quote(base.span), name.text);
        let end = Span::new(expr.span.end, expr.span.end);
        self.sink.push(
            cpl_diag::msgs::check::PARENS_MEAN_WORK
                .warning(expr.span, &[&shown])
                .with_help(habits::SUM.say(&[&shown]))
                .with_fix(habits::SUM_2.say(&[&shown]), end, "()"),
        );
    }

    /// `a * inv` with no `impl Mul` for `inv`'s type. When `inv` is a
    /// function not yet called (`x.inv`) whose result fits, the fix calls it.
    pub(super) fn no_operator_impl(
        &mut self,
        (op, op_span): (BinOp, Span),
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
                    cpl_diag::msgs::check::TAKES_BUT_METHOD_THAT.at(&[&o, &ret, &name])
                } else {
                    cpl_diag::msgs::check::TAKES_BUT_HOLDS_METHOD.at(&[&o, &ret, &name])
                };
                if let Some(diag) = self.report_said(op_span, message) {
                    if inline {
                        diag.help = Some(habits::NO_OPERATOR_IMPL.say(&[&ret, &name]));
                        let end = Span::new(rhs.span.end, rhs.span.end);
                        diag.fix(habits::SUM_2.say(&[&name]), end, "()");
                    } else {
                        diag.help = Some(habits::NO_OPERATOR_IMPL_2.say(&[&name]));
                    }
                }
                return Ty::Error;
            }
        }
        let message =
            cpl_diag::msgs::check::NEEDS_IMPL_THERE_NONE.at(&[&o, &trait_name, &right, &owner]);
        if let Some(diag) = self.report_said(op_span, message) {
            diag.help = Some(habits::NO_OPERATOR_IMPL_3.say(&[
                &trait_name,
                &right,
                &owner,
                &owner,
                &o,
                &right,
            ]));
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
        if let Some(diag) = self.report(pat.span, msgs::check::TYPE_MISMATCH_EACH_RUN_IS_A, &[]) {
            let pair = ".map(r => (r.first!, r.len))";
            diag.help = Some(habits::LOOP_ADVICE.say(&[&pair]));
            let end = Span::new(iter.span.end, iter.span.end);
            diag.fix(habits::WRITE.say(&[&pair]), end, pair);
        }
    }

    /// The trait in scope that declares member `name`, where `owner` does not
    /// implement it: `.len` of a filtered view is `Sized`'s.
    fn unmet_member_trait(&mut self, owner: &str, name: &str) -> Option<String> {
        // A type with no trait impls takes no trait's members (R08.1.7).
        if !self.env.impls(owner).iter().any(|(t, _)| t.is_some()) {
            return None;
        }
        // An impl lives with its trait or its type (R08.1.2): one neither
        // is this file's can't be written here, so the members are told.
        let own_type = self.local_defs.contains_key(owner) || self.env.declares_type(owner);
        let prelude = crate::prelude::names().iter().map(|(_, n)| n.text.clone());
        let names: Vec<String> = self.env.names().into_iter().chain(prelude).collect();
        for trait_name in names {
            let Some(Def::Trait {
                methods,
                alias: false,
                ..
            }) = self.type_def(&trait_name).as_deref().cloned()
            else {
                continue;
            };
            if methods.iter().any(|m| m.name == name)
                && (own_type
                    || self.local_defs.contains_key(&trait_name)
                    || self.env.decl_order(&trait_name).is_some())
                && !self.has_impl(owner, &trait_name)
            {
                return Some(trait_name);
            }
        }
        None
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
        if matches!(base_ty, Ty::Named { .. }) {
            if let Some(trait_name) = self.unmet_member_trait(&owner, &name.text) {
                self.report(
                    name.span,
                    msgs::check::MISSING_IMPL_MEMBER_REQUIRES_WHICH,
                    &[&name.text, &trait_name, base_ty],
                );
                return;
            }
        }
        let public =
            |c: &mut Self, m: &String| owner.is_empty() || !c.env.is_private_field(&owner, m);
        let mut public_fields = Vec::new();
        for f in field_names {
            if public(self, f) {
                public_fields.push(f.clone());
            }
        }
        let field_names = public_fields;
        let mut public_methods = Vec::new();
        for m in methods {
            if public(self, &m.0) {
                public_methods.push(m.clone());
            }
        }
        let methods = public_methods;
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
        // C++'s `pq.top()`: a heap's top is its `peek`, never the `pop` it's spelled like.
        let top = name.text == "top" && has("peek") && after.starts_with("()");
        let unwrap =
            matches!(base_ty, Ty::Optional(_)) && matches!(name.text.as_str(), "unwrap" | "expect");
        let peek_getter = |getters: &[String]| getters.iter().any(|g| g == "peek");
        let peek = match self.type_def(&owner).as_deref() {
            Some(Def::Struct { getters, .. }) if peek_getter(getters) => "peek!",
            _ if (self.env.impls(&owner).iter()).any(
                |(_, def)| matches!(&**def, Def::Impl { getters, .. } if peek_getter(getters)),
            ) =>
            {
                "peek!"
            }
            _ => "peek()!",
        };
        let mut known: Vec<&str> = Vec::new();
        for f in &field_names {
            known.push(f);
        }
        for (m, _) in &methods {
            known.push(m);
        }
        if has("range_first") {
            known.push("range");
        }
        let suggestion = closest(&name.text, known.iter().copied());
        let library_domain = match base_ty {
            Ty::Named { name, .. } | Ty::TypeName(name) => self.has_impl(name, "Index"),
            _ => false,
        };
        let core_only =
            library_domain && super::library_domain::CORE_DOMAIN_ONLY.contains(&name.text.as_str());
        let said = if unwrap {
            msgs::check::NO_METHOD_ON.at(&[&"a", base_ty, &name.text])
        } else {
            msgs::check::UNKNOWN_FIELD_HAS_NO_FIELD_OR.at(&[base_ty, &name.text])
        };
        let Some(diag) = self.report_said(name.span, said) else {
            return;
        };
        let renamed = library_domain
            .then(|| super::library_domain::index_rename(&name.text))
            .flatten();
        match suggestion {
            _ if renamed.is_some() => {
                let new = renamed.unwrap_or_default();
                diag.help = Some(habits::UNKNOWN_MEMBER.say(&[&name.text, &new]));
                diag.fix(habits::WRITE.say(&[&new]), name.span, new);
            }
            _ if core_only => {
                diag.help = Some(habits::UNKNOWN_MEMBER_2.say(&[&base_ty, &base_ty]));
            }
            _ if habit.is_some() => {
                let own = habit.unwrap_or_default();
                diag.help = Some(habits::UNKNOWN_MEMBER_3.say(&[&name.text, &own]));
                diag.fix(habits::WRITE.say(&[&own]), name.span, own);
            }
            _ if empty_by_len => {
                diag.help = Some(habits::UNKNOWN_MEMBER_4.say(&[&base_ty]));
                diag.fix(habits::UNKNOWN_MEMBER_5.say(&[]), name.span, "len == 0");
            }
            _ if top => {
                diag.help = Some(habits::UNKNOWN_MEMBER_6.say(&[&base_ty]));
                let at = Span::new(name.span.start, name.span.end + 2);
                diag.fix(habits::WRITE.say(&[&peek]), at, peek);
            }
            // `x.unwrap()` is `x!`.
            _ if unwrap => {
                diag.help = Some(crate::builtins::OPTIONAL_UNWRAP_HELP.to_string());
                if after.starts_with("()") {
                    let at = Span::new(name.span.start.saturating_sub(1), name.span.end + 2);
                    diag.fix(habits::WRITE.say(&[&"!"]), at, "!");
                }
            }
            _ if fold_for => {
                diag.help = Some(habits::UNKNOWN_MEMBER_7.say(&[&base_ty]));
                diag.fix(habits::WRITE.say(&[&"fold"]), name.span, "fold");
            }
            Some(s) => diag.suggest(name.span, s),
            None if matches!(base_ty, Ty::Tuple(_)) => {
                let Ty::Tuple(parts) = base_ty else { return };
                diag.help = Some(habits::TUPLE_PARTS.say(&[&parts.len().saturating_sub(1)]));
            }
            None if !field_names.is_empty() => {
                let fields = super::expr::list_names(&field_names);
                diag.help = Some(habits::UNKNOWN_MEMBER_9.say(&[&fields]));
            }
            None => {
                let names = self
                    .public_receivers(&owner)
                    .unwrap_or_else(|| methods.iter().map(|(m, _)| m.clone()).collect());
                if let Some(list) = method_list(&names) {
                    if let Some(d) = self.sink.last_mut() {
                        d.help = Some(habits::UNKNOWN_MEMBER_10.say(&[&list]));
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
        let def = self.type_def(owner)?;
        let Def::Struct {
            receivers,
            mutators,
            getters,
            ..
        } = &*def
        else {
            return None;
        };
        let mut names = Vec::new();
        for m in [receivers, mutators, getters].into_iter().flatten() {
            if !self.env.is_private_field(owner, m) {
                names.push(m.clone());
            }
        }
        Some(names)
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
    /// The `None` literal, whose optional the context gives.
    pub(super) fn none_literal(&mut self, span: Span) -> Ty {
        // `None` checks as anything; its fact is an optional of some type.
        (self.exprs.types).push((span, Ty::Optional(Box::new(Ty::Unknown))));
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
        if let Some(diag) = self.report(
            span,
            msgs::check::REMOVED_SYNTAX_IS_SPELLED_SINCE_0,
            &[&old, &new],
        ) {
            diag.help = Some(habits::OLD_OPTIONAL_SPELLING.say(&[&old]));
            diag.fix(habits::WRITE.say(&[&new]), span, new);
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
            d.help = Some(habits::REVERSE_VIEW_HELP.say(&[&help, &copy]));
        }
    }

    /// `enumerate(xs)`, `reversed(xs)`, `pow(a, b)`, `s.charAt(i)`: foreign
    /// functions for what CP Lang spells as a member or an operator, so the
    /// unresolved name gets `xs.items` or `a ** b`.
    pub(super) fn member_for_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) {
        if self.removed_call(call, callee, args) || self.receiver_free_fn(call, callee, args) {
            return;
        }
        let (name, at, code) = match &callee.kind {
            ExprKind::Name(name) => (name, callee.span, codes::UNRESOLVED_NAME),
            ExprKind::Field { name, .. } => (&name.text, name.span, codes::UNKNOWN_FIELD),
            _ => return,
        };
        // `max_element(a.begin(), a.end())`: the iterator pair's fix rewrites
        // the whole call, so the unknown callee's error carries it alone.
        let rewrites_call = |d: &cpl_diag::Diagnostic| {
            d.code == codes::UNKNOWN_FIELD
                && call.span.contains(d.span)
                && d.fixes
                    .iter()
                    .any(|f| f.edits.iter().any(|(s, _)| s.start == call.span.start))
        };
        let callee_error = |d: &cpl_diag::Diagnostic| d.code == code && d.span == at;
        let diags = self.sink.diagnostics();
        let pair = diags
            .iter()
            .find(|d| rewrites_call(d))
            .map(|d| d.fixes.clone());
        let unknown = diags.iter().find(|d| callee_error(d)).cloned();
        if let (Some(fixes), Some(mut diag), codes::UNRESOLVED_NAME) = (pair, unknown, code) {
            self.sink
                .drop_where(|d| rewrites_call(d) || callee_error(d));
            diag.fixes = fixes;
            self.sink.push(diag);
        }
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
        if let ExprKind::Field { base, name: f } = &callee.kind {
            // Python's `heapq.heappush(h, x)`: a list pushed to is no heap, so
            // the fix needs `h` to be a type with heapq's `heappushpop`, `push_pop`.
            if matches!(&base.kind, ExprKind::Name(m) if m == "heapq") {
                let heap = |c: &mut Self, a: &Arg| {
                    let ty = c.infer_quietly(&a.value);
                    matches!(&ty, Ty::Named { name, .. }
                        if c.struct_method(name, "push_pop").is_some())
                };
                let ours = match (f.text.as_str(), args) {
                    ("heappush", [h, x]) if heap(self, h) => Some(format!(
                        "{}.push({})",
                        self.text(h.value.span),
                        self.text(x.value.span)
                    )),
                    ("heappop", [h]) if heap(self, h) => {
                        Some(format!("{}.pop()!", self.text(h.value.span)))
                    }
                    _ => None,
                };
                let member = f.span;
                self.sink
                    .drop_where(|d| d.code == codes::UNKNOWN_FIELD && d.span == member);
                let unresolved = |d: &&mut cpl_diag::Diagnostic| {
                    d.code == codes::UNRESOLVED_NAME && d.span == base.span
                };
                if let Some(diag) = self.sink.last_mut().filter(unresolved) {
                    diag.fixes.clear();
                    diag.help = Some(match &ours {
                        Some(new) => habits::HEAP_METHODS.say(&[new]),
                        None => habits::HEAP.say(&[]),
                    });
                    if let Some(new) = ours {
                        diag.fix(habits::WRITE.say(&[&new]), call.span, new);
                    }
                }
                return;
            }
            if matches!(&base.kind, ExprKind::Name(m) if m == "math" || m == "Math") {
                // `math.ceil(a / b)` on Ints: `/` already rounds down.
                let div = match (f.text.as_str(), args) {
                    ("ceil" | "floor", [x]) => match &x.value.kind {
                        ExprKind::Binary {
                            op: BinOp::Div,
                            lhs,
                            rhs,
                            ..
                        } => {
                            let l = self.infer_quietly(lhs);
                            let r = self.infer_quietly(rhs);
                            let (a, b) = (self.text(lhs.span), self.text(rhs.span));
                            let bare = !a.contains(' ') || a.starts_with('(') && a.ends_with(')');
                            let a = if bare {
                                a.to_string()
                            } else {
                                format!("({a})")
                            };
                            (l.is_int() && r.is_int()).then(|| (a, b.to_string()))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                let ours = match (f.text.as_str(), args, div) {
                    ("ceil", [_], Some((a, b))) => format!("{a}.ceil_div({b})"),
                    ("floor", [_], Some((a, b))) => format!("{a} / {b}"),
                    ("sqrt" | "abs" | "floor" | "ceil" | "round" | "trunc" | "isqrt", [x], _) => {
                        format!("{}.{}", operand(self, x), f.text)
                    }
                    ("gcd" | "lcm", [a, b], _) => {
                        let (a, b) = (self.text(a.value.span), self.text(b.value.span));
                        format!("{}({a}, {b})", f.text)
                    }
                    ("pow", [a, b], _) => format!("{} ** {}", operand(self, a), operand(self, b)),
                    _ => return,
                };
                let member = f.span;
                self.sink
                    .drop_where(|d| d.code == codes::UNKNOWN_FIELD && d.span == member);
                let unresolved = |d: &&mut cpl_diag::Diagnostic| {
                    d.code == codes::UNRESOLVED_NAME && d.span == base.span
                };
                if let Some(diag) = self.sink.last_mut().filter(unresolved) {
                    diag.help = Some(habits::MEMBER_FOR_CALL.say(&[&ours]));
                    diag.fixes.clear();
                    diag.fix(habits::WRITE.say(&[&ours]), call.span, ours);
                }
                return;
            }
        }
        let arg_ty = match args {
            [a] => self.hovers.iter().rev().find(|h| h.span == a.value.span),
            _ => None,
        }
        .map(|h| h.ty.clone());
        // `list(map(int, input().split()))`: Python reads a line of numbers;
        // CP Lang reads tokens, and how many is the contestant's call.
        let numbers = |c: &Self, a: &[Arg]| {
            matches!(a, [f, xs] if matches!(c.text(f.value.span), "int" | "float")
                && c.text(xs.value.span).contains("input()"))
        };
        if name == "map" && numbers(self, args) {
            let whole = call.span;
            self.sink.drop_where(|d| {
                d.code == codes::UNRESOLVED_NAME && whole.contains(d.span) && d.span != at
            });
            if let Some(diag) = self
                .sink
                .last_mut()
                .filter(|d| d.code == code && d.span == at)
            {
                diag.help = Some(habits::MEMBER_FOR_CALL_2.say(&[]));
                diag.fixes.clear();
            }
            return;
        }
        let mapped = |c: &Self, x: &Arg| match &x.value.kind {
            ExprKind::Call { callee, args } => {
                matches!(&callee.kind, ExprKind::Name(m) if m == "map") && numbers(c, args)
            }
            _ => false,
        };
        if name == "list" && matches!(args, [x] if mapped(self, x)) {
            self.sink.drop_where(|d| d.code == code && d.span == at);
            return;
        }
        // `" ".join(map(str, a))`: `join` shows each item, so `a` is enough.
        let joined = |c: &Self| {
            let before = Span::new(call.span.start.saturating_sub(6), call.span.start);
            let after = Span::new(call.span.end, call.span.end + 1);
            c.text(before) == ".join(" && c.text(after) == ")"
        };
        let shown = match (name.as_str(), args) {
            ("map", [f, xs]) if self.text(f.value.span) == "str" && joined(self) => {
                let f = f.value.span;
                self.sink
                    .drop_where(|d| d.code == codes::UNRESOLVED_NAME && d.span == f);
                Some(self.text(xs.value.span).to_string())
            }
            _ => None,
        };
        let (ours, what) = match (name.as_str(), args) {
            ("map", [_, _]) if shown.is_some() => (
                shown.unwrap_or_default(),
                "`join` shows each item itself, so it takes the list",
            ),
            ("int", [x]) if matches!(arg_ty, Some(Ty::Float)) => (
                format!("{}.trunc", operand(self, x)),
                "a float rounds toward zero with `trunc` (or `floor`, `round`)",
            ),
            ("int", [c]) if matches!(arg_ty, Some(Ty::Char)) => (
                format!("{}.digit", operand(self, c)),
                "a digit's value is its `digit`",
            ),
            ("float", [s]) if infinity(self.text(s.value.span)).is_some() => (
                infinity(self.text(s.value.span))
                    .unwrap_or("inf")
                    .to_string(),
                "an infinity is the largest `Int`",
            ),
            ("int" | "float", [_]) if input.is_some() => (
                format!("read<{}>()", read.unwrap_or("Int")),
                "a value is read from the input with `read`",
            ),
            ("int" | "float" | "stoi" | "stoll" | "atoi", [s]) if input.is_none() => (
                format!("{}.parse<{}>()!", operand(self, s), read.unwrap_or("Int")),
                "a number is read from a string with `parse`",
            ),
            ("range", [a, b, step]) => {
                let (a, b) = (self.text(a.value.span), self.text(b.value.span));
                let k = self.text(step.value.span);
                match k.strip_prefix('-').map(str::trim) {
                    // `range(a, b, -k)` walks down from `a` while above `b`.
                    Some(k) => {
                        let lo = match b.parse::<i64>() {
                            Ok(b) => (b + 1).to_string(),
                            Err(_) => format!("{b} + 1"),
                        };
                        let walk = match a.strip_suffix(" - 1") {
                            Some(hi) => format!("({lo}..{hi}).rev"),
                            None => format!("({lo}..={a}).rev"),
                        };
                        let walk = match k {
                            "1" => walk,
                            k => format!("{walk}.step({k})"),
                        };
                        (walk, "a range walked down is its `rev`")
                    }
                    None => (
                        format!("({a}..{b}).step({k})"),
                        "a range with a step is written",
                    ),
                }
            }
            ("dict", []) => ("Map()".to_string(), "an empty map is written"),
            // A missing key already reads as 0 under `+=`.
            ("defaultdict", [t]) if matches!(self.text(t.value.span), "int" | "float") => {
                let at = t.value.span;
                self.sink
                    .drop_where(|d| d.code == codes::UNRESOLVED_NAME && d.span == at);
                (
                    "Map()".to_string(),
                    "a `defaultdict(int)` is a plain map, whose missing keys read as 0,",
                )
            }
            ("input", []) => (
                "read_line()".to_string(),
                "a line is read with `read_line` (a value with `read()`)",
            ),
            ("memset" | "fill", [a, v]) => (
                format!("{}.fill({})", operand(self, a), self.text(v.value.span)),
                "a list is filled with its `fill`",
            ),
            ("make_pair" | "make_tuple" | "tuple", [_, _, ..]) => {
                let items: Vec<&str> = args.iter().map(|a| self.text(a.value.span)).collect();
                (
                    format!("({})", items.join(", ")),
                    "a tuple is written in parentheses",
                )
            }
            ("__builtin_popcount" | "__builtin_popcountll", [x]) => (
                format!("popcount({})", self.text(x.value.span)),
                "the 1 bits are counted with `popcount`",
            ),
            ("bisect_left" | "bisect_right", [xs, x]) => {
                let m = if name == "bisect_left" {
                    "first_ge"
                } else {
                    "first_gt"
                };
                let xs = operand(self, xs);
                (
                    format!("({xs}.{m}({}) ?? {xs}.len)", self.text(x.value.span)),
                    "a sorted list is searched with `first_ge` and `first_gt`",
                )
            }
            ("enumerate", [xs]) => (
                format!("{}.items", operand(self, xs)),
                "index-element pairs are a member",
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
            ("string", [n, c]) if self.text(c.value.span).starts_with('\'') => {
                let c = self.text(c.value.span);
                (
                    format!(
                        "\"{}\".repeat({})",
                        match &c[1..c.len() - 1] {
                            "\"" => "\\\"",
                            "\\'" => "'",
                            ch => ch,
                        },
                        self.text(n.value.span)
                    ),
                    "a repeated string is written with `repeat`",
                )
            }
            ("str" | "to_string", [x]) if simple(self.text(x.value.span)) => (
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
        let Some(diag) = self.sink.last_mut().filter(|d| {
            // `pow(2, 10, m)` reaches std's `pow`, whose `op:` is named-only.
            let miscounted = d.code == codes::WRONG_ARG_COUNT && d.span == call.span;
            (d.code == code && d.span == at || miscounted) && args.iter().all(|a| a.name.is_none())
        }) else {
            return;
        };
        diag.help = Some(habits::MEMBER_FOR_CALL_3.say(&[&what, &ours]));
        diag.fixes.clear();
        diag.fix(habits::WRITE.say(&[&ours]), call.span, ours);
    }

    /// `"a"` where a `Char` is wanted: a character is in single quotes.
    pub(super) fn char_literal_fix(&mut self, got: &Ty, want: &Ty, at: &Expr) -> bool {
        if let (Ty::Str, Ty::Char, Some(c)) = (got, want, crate::char_literal(self.text(at.span))) {
            if let Some(diag) = self.sink.last_mut().filter(|d| d.span == at.span) {
                diag.help = Some(habits::CHAR_LITERAL_FIX.say(&[&c]));
                diag.fix(habits::WRITE.say(&[&c]), at.span, c);
            }
        }
        false
    }

    /// `x in xs` on a list: `in` is sub-linear only, so a scan is spelled
    /// out as `xs.contains(x)`.
    /// Whether `x in s` misses `Contains` (R12.4.7): a named type that neither implements it nor
    /// declares a `contains` or `has` of its own, such as a list, a deque or a user `Seq`.
    pub(super) fn lacks_contains(&mut self, ty: &Ty) -> bool {
        matches!(ty, Ty::Named { .. })
            && !self.implements_trait(ty, "Contains")
            && !self.declares_member(ty, "contains")
            && !self.declares_member(ty, "has")
    }

    pub(super) fn list_in(&mut self, op: CmpOp, x: &Expr, xs: &Expr, ty: &Ty, alone: bool) {
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
        if let Some(diag) = self.report(
            xs.span,
            msgs::check::MISSING_IMPL_IN_REQUIRES_CONTAINS_WHICH,
            &[&ty],
        ) {
            diag.help = Some(habits::LIST_IN.say(&[]));
            if alone {
                diag.fix(habits::LIST_IN_2.say(&[&scan]), whole, scan);
            }
        }
    }

    /// `var m: Map<K, V> = {}` or `var d = {}`: braces make a block, so the empty container
    /// is reported as one, with its spelling as the fix.
    pub(super) fn empty_braces(&mut self, declared: Option<&Ty>, init: Option<&Expr>) {
        let Some(init) = init else { return };
        let (empty, kind, help) = match declared {
            Some(d @ Ty::Named { name, .. }) if name == "Map" => {
                ("[:]".into(), d.to_string(), &habits::EMPTY_BRACES)
            }
            Some(d @ Ty::Named { name, .. }) if name == "Set" => {
                ("Set()".into(), d.to_string(), &habits::EMPTY_BRACES)
            }
            Some(d @ Ty::Named { name, .. }) if name == "List" => {
                ("[]".into(), d.to_string(), &habits::EMPTY_BRACES)
            }
            // Decision 220: `{}` is no Subset literal; the empty subset is `Subset<B>.empty`,
            // as for any struct std gives an `empty`.
            Some(d @ Ty::Named { name, .. })
                if self
                    .members_of(name)
                    .is_some_and(|m| m.members.iter().any(|(m, _)| m == "empty")) =>
            {
                (format!("{d}.empty"), d.to_string(), &habits::EMPTY_BRACES)
            }
            // `d = {}` with no type: Python's empty dict.
            None => ("Map()".to_string(), "map".to_string(), &habits::EMPTY_DICT),
            _ => return,
        };
        if self.text(init.span).split_whitespace().collect::<String>() != "{}" {
            return;
        }
        let reported = |c: &mut Self| c.sink.last_mut().is_some_and(|d| d.span == init.span);
        if declared.is_none() && !reported(self) {
            self.report(
                init.span,
                cpl_diag::msgs::check::EMPTY_BLOCK_NOT_EMPTY,
                &[&kind],
            );
        }
        let Some(diag) = self.sink.last_mut().filter(|d| d.span == init.span) else {
            return;
        };
        diag.message = cpl_diag::msgs::check::EMPTY_BLOCK_NOT_EMPTY.say(&[&"{}", &kind]);
        diag.help = Some(help.say(&[&kind, &empty]));
        diag.fix(habits::WRITE.say(&[&empty]), init.span, empty.clone());
    }

    /// A member that another version or language has and CP Lang spells
    /// differently: reported, with the fix where there is one.
    pub(super) fn member_advice(&mut self, base_ty: &Ty, name: &cpl_syntax::ast::Name) -> bool {
        if let Some(new) = super::library_domain::index_rename(&name.text) {
            if self.index_like(base_ty, &name.text) {
                // `from_idx` gave an optional; `try_at` keeps that, `nth` traps instead.
                let fix = if new == "nth" { "try_at" } else { new };
                if let Some(diag) = self.report(
                    name.span,
                    msgs::check::REMOVED_SYNTAX_WAS_RENAMED_TO_IN,
                    &[&name.text, &new],
                ) {
                    diag.help = Some(habits::MEMBER_ADVICE.say(&[]));
                    diag.fix(habits::WRITE.say(&[&fix]), name.span, fix);
                }
                return true;
            }
        }
        if name.text == "bounds" && matches!(base_ty, Ty::Domain(_)) {
            if let Some(diag) = self.report(
                name.span,
                msgs::check::REMOVED_SYNTAX_D_BOUNDS_WAS_RENAMED,
                &[],
            ) {
                diag.fix(habits::WRITE.say(&[&"cuts"]), name.span, "cuts");
            }
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
        if let Some(diag) = self.report(
            name.span,
            msgs::check::UNKNOWN_FIELD_A_TUPLE_HAS_NO,
            &[&name.text],
        ) {
            diag.help = Some(habits::MEMBER_ADVICE_3.say(&[&last]));
            diag.fix(habits::MEMBER_ADVICE_4.say(&[&i]), name.span, i.to_string());
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
            diag.help = Some(habits::ITER_CALL_HELP.say(&[]));
            diag.fixes.clear();
            diag.fix(habits::ITER_CALL_HELP_2.say(&[&name.text]), call, "");
        }
    }

    /// `.len` on a lazy sequence, which has no stored length.
    pub(super) fn lazy_len(&mut self, base_ty: &Ty, name: &Name) {
        if let Some(diag) = self.report(
            name.span,
            msgs::check::MISSING_IMPL_LEN_REQUIRES_SIZED_WHICH,
            &[&base_ty],
        ) {
            diag.help = Some(habits::LAZY_LEN.say(&[]));
            diag.fix(habits::WRITE.say(&[&".count()"]), name.span, "count()");
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
            title: habits::LITERAL.say(&[]),
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
        if let Some(diag) = self.report(
            value.span,
            msgs::check::TYPE_MISMATCH_RETURNS_A_VALUE_BUT,
            &[&name],
        ) {
            diag.help = Some(habits::UNDECLARED_RETURN.say(&[&ty]));
            if let Some(slot) = slot {
                diag.fix(habits::WITHOUT.say(&[&ty]), slot, format!(" -> {ty}"));
            }
        }
    }
}

/// E0601's message and help: one place passed twice is said plainly.
pub(super) fn lent_twice(a: &Arg, b: &Arg, why: &str) -> (cpl_diag::msgs::Said, String) {
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
            cpl_diag::msgs::check::LENT_TWICE_TO_CALL.at(&[&first, &how, &reason]),
            format!("pass `{first}` once"),
        );
    }
    let how = if a.mutable && b.mutable {
        "both are"
    } else {
        "one is"
    };
    (
        cpl_diag::msgs::check::MAY_BE_SAME_STORAGE.at(&[&first, &second, &how, &why]),
        "guard it (`if i != j: ...`), state it (`assert i != j`), or copy the read-only one first"
            .into(),
    )
}

/// E0407: point at the first variant with the tag, and say what to change.
pub(super) fn duplicate_tag(diag: &mut Diagnostic, first: Span) {
    diag.labels.push(cpl_diag::Label {
        span: first,
        message: "first `@tag` here".to_string(),
    });
    diag.help = Some(habits::DUPLICATE_TAG.say(&[]));
}

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
        let listed = crate::builtins::BUILTIN_MEMBERS.iter().any(|m| {
            m.receiver == crate::builtins::Receiver::List
                && m.mutates
                && m.params.is_empty()
                && m.name == stem
        });
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
    std_in_place_method_in(crate::modules::std_index(), stem)
}

fn std_in_place_method_in(std: crate::prelude::Std, stem: &str) -> Option<(String, String)> {
    for (module, text) in std {
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

/// Whether the text after a member is its call: `(..)`, or `<T>(..)`
/// as in `s.parse<Int>()`.
pub(super) fn called_next(after: &str) -> bool {
    let mut rest = after;
    if let Some(inner) = after.strip_prefix('<') {
        let mut depth = 1;
        let close = inner.find(|c| {
            depth += i32::from(c == '<') - i32::from(c == '>');
            depth == 0
        });
        if let Some(i) = close {
            rest = &inner[i + 1..];
        }
    }
    rest.trim_start().starts_with('(')
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
    let word = |w: &str| {
        src.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|at| at == w)
    };
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
        let taken = !d.fixes.is_empty() || word(&lower);
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
        cpl_diag::sort_by_key(&mut edits, |(s, _)| s.start as u64);
        edits.dedup_by_key(|(s, _)| s.start);
        let title = habits::RENAME_VALUES.say(&[&old, &lower]);
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
) -> Diagnostic {
    use super::lends::show;
    use cpl_diag::msgs::check::{PART_OF, SAME_PLACE};
    let (changed, assigned) = (show(place), show(target));
    let why = match parts {
        std::cmp::Ordering::Greater => PART_OF.say(&[&assigned, &changed]),
        std::cmp::Ordering::Less => PART_OF.say(&[&changed, &assigned]),
        std::cmp::Ordering::Equal => why,
    };
    let lends_mut =
        matches!(&value.kind, ExprKind::Call { args, .. } if args.iter().any(|a| a.mutable));
    let first = match op {
        cpl_syntax::ast::AssignOp::Set if !lends_mut => {
            format!("first, `let v = {}`, then `{assigned} = v`", show(value))
        }
        _ => "into a `let` first, then assign".into(),
    };
    SAME_PLACE
        .warning(place.span, &[&changed, &assigned, &why])
        .with_help(habits::SAME_PLACE.say(&[&first]))
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
                    let base_ty = self.infer_quietly(base);
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

pub(super) fn method_call<'e>(e: &'e Expr, method: &str) -> Option<(&'e Expr, &'e [Arg])> {
    match &e.kind {
        ExprKind::Call { callee, args } => match &callee.kind {
            ExprKind::Field { base, name } if name.text == method => Some((base, args)),
            _ => None,
        },
        _ => None,
    }
}

impl Checker<'_> {
    /// An enum's tokens are a distinct `@tag` on each variant; `@tags("U",
    /// "D")` above the enum is removed (R02.7.2, decision 459).
    pub(super) fn enum_tags(
        &mut self,
        stmt: &cpl_syntax::ast::Stmt,
        e: &cpl_syntax::ast::EnumDecl,
    ) {
        let mut seen: Vec<(String, Span)> = Vec::new();
        for tag in e.variants.iter().filter_map(|v| v.tag.as_ref()) {
            let text = self.quote(tag.span);
            let Some((_, prev)) = seen.iter().find(|(t, _)| *t == text) else {
                seen.push((text, tag.span));
                continue;
            };
            let prev = *prev;
            if let Some(diag) = self.report(
                tag.span,
                msgs::check::DUPLICATE_VARIANT_TAG_ENUM_ALREADY_HAS_A,
                &[&e.name.text, &text],
            ) {
                duplicate_tag(diag, prev);
            }
        }
        let Some(tags) = stmt.decorators.iter().find(|d| d.name.text == "tags") else {
            return;
        };
        let each: Vec<String> = (e.variants.iter().zip(&tags.args))
            .map(|(v, tag)| format!("@tag({}) {}", self.quote(tag.value.span), v.name.text))
            .collect();
        let message = cpl_diag::msgs::check::ENUM_LEVEL_TAGS_WAS.at(&[]);
        let Some(diag) = self.report_said(tags.span, message) else {
            return;
        };
        if each.len() == e.variants.len() && tags.args.len() == each.len() {
            diag.help = Some(habits::ENUM_TAGS.say(&[&e.name.text, &each.join(" | ")]));
        }
    }
}

#[cfg(test)]
mod tests {
    /// The in-place hint reads std's outlines and finds what the files say.
    #[test]
    fn the_in_place_hint_reads_the_same_off_the_outlines() {
        let (files, index) = (crate::modules::std_modules(), crate::modules::std_index());
        let found = |std, stem| super::std_in_place_method_in(std, stem);
        assert_eq!(found(index, "shuffle"), Some(("Rng".into(), "rand".into())));
        for name in crate::prelude::spec_member_names() {
            assert_eq!(found(files, name), found(index, name), "{name}");
        }
    }
}
