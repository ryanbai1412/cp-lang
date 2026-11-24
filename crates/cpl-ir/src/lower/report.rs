//! Where a lowering error is shown, and what it says when it is inside
//! library code.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// Where an error at `span` is shown. One inside library code is
    /// shown at the user's expression that asked for that code, since the
    /// file on screen is the user's.
    pub(super) fn span(&self, span: Span) -> Span {
        if self.cur_mod == 0 {
            span.shift(self.offset)
        } else {
            self.asked_at.at
        }
    }

    /// A call's result is a new value, never a place, so it can't be `mut`:
    /// `absorb(win[r].take())` means `absorb(mut win[r])`, which empties it.
    pub(super) fn temporary_mut_help(&mut self, arg: &ast::Expr) {
        let ast::ExprKind::Call { callee, args } = &arg.kind else {
            return;
        };
        let span = self.span(arg.span);
        let text = self.source(arg.span);
        let new_value =
            format!("`{text}` is a new value, not a variable, so it can never be `mut`");
        let (help, fix) = match &callee.kind {
            ast::ExprKind::Field { base, name } if name.text == "take" && args.is_empty() => {
                let place = format!("mut {}", self.source(base.span));
                (
                    format!(
                        "{new_value}; pass the place itself, `{place}`, which the call empties"
                    ),
                    Some(place),
                )
            }
            _ => (
                format!("{new_value}; store it with `var` first and pass that with `mut`"),
                None,
            ),
        };
        let Some(diag) = self.diags.last_mut() else {
            return;
        };
        diag.help = Some(help);
        if let Some(fix) = fix {
            diag.fix(format!("write `{fix}`"), span, fix);
        }
    }

    /// An assert's condition as written, the line it reports, and where it
    /// is when that is a library: there it reports the user's line that
    /// reached it, and names its own line in a note.
    pub(super) fn assert_place(&self, cond: Span) -> (String, u32, Option<String>) {
        let text = self.source(cond);
        if self.cur_mod == 0 {
            return (text, self.line(cond), None);
        }
        let line = self.mods[0].lines.line_of(self.asked_at.at.start) + 1;
        let place = self.library_place(self.cur_mod, cond.shift(self.offset));
        let note = place.map(|(module, at, inside)| {
            format!("{}.cp:{at}: inside `{inside}`", module.replace('.', "/"))
        });
        (text, line, note)
    }

    pub(super) fn report(
        &mut self,
        span: Span,
        msg: cpl_diag::msgs::Msg,
        args: &[&dyn std::fmt::Display],
    ) {
        self.report_said(span, msg.at(args));
    }

    /// Deletes `cut` (an AST span) from the user's code as the last error's
    /// fix: the `!` or `?? d` on a value that is never `None`.
    pub(super) fn drop_fix(&mut self, title: &str, cut: Span) {
        if self.cur_mod != 0 {
            return;
        }
        let cut = self.span(cut);
        if let Some(diag) = self.diags.last_mut() {
            diag.fix(title, cut, "");
        }
    }

    pub(super) fn report_said(&mut self, span: Span, mut said: cpl_diag::msgs::Said) {
        let at = span.shift(self.offset);
        let span = self.span(span);
        let mut notes = Vec::new();
        if self.cur_mod != 0 {
            if let Some((module, line, inside)) = self.library_place(self.cur_mod, at) {
                let inside = match inside.split_once('.') {
                    Some((ty, f)) => format!("{}.{f}", cpl_check::written_name(ty)),
                    None => inside,
                };
                said = said.wrapped(msgs::ir::LIBRARY_GAP, &[&inside, &module]);
                notes.push(format!(
                    "{}.cp:{line}: inside `{inside}`; the gap is in `{module}`, not in your code",
                    module.replace('.', "/")
                ));
            }
            for &(module, call) in self.asked_at.via.iter().rev() {
                if let Some((module, line, inside)) = self.library_place(module, call) {
                    let file = module.replace('.', "/");
                    notes.push(format!("{file}.cp:{line}: called from `{inside}`"));
                }
            }
            self.in_library.insert(said.as_str().to_string());
        }
        let diag = said.error(span);
        self.diags
            .push(notes.into_iter().fold(diag, Diagnostic::with_note));
    }

    /// The type a generic alias such as `Perm<V>` names, resolved where
    /// the alias is declared; an error inside it is shown at `at`.
    pub(super) fn gen_alias_spec(
        &mut self,
        gaid: mono::GenAliasId,
        args: &[ast::Type],
        at: Span,
        values: bool,
    ) -> Spec {
        let gargs = self.resolve_gen_args(args);
        let alias = &self.gen_aliases[gaid];
        let (tpl_mod, tpl_offset, target_ty) = (alias.mod_id, alias.offset, alias.ty);
        let env = alias
            .generics
            .iter()
            .map(|p| p.text.clone())
            .zip(gargs)
            .collect();
        let outer = self.asked_at.enter(at.shift(self.offset), self.cur_mod);
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let spec = self.resolve(target_ty, values);
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.asked_at.leave(outer);
        spec
    }

    /// `[V]T` in a generic whose `V` was given `arg`, which is not a
    /// domain, so it has no positions to index by. The argument is the
    /// user's, so this is not reported as a gap in the library.
    pub(super) fn not_a_domain(
        &mut self,
        param: &str,
        arg: &GenArg,
        span: Span,
    ) -> Option<DimSpec> {
        let what = match arg {
            GenArg::Ty(ty) => format!("`{}`", self.show(ty)),
            _ => "a range".to_string(),
        };
        let inside = (self.cur_mod != 0)
            .then(|| self.library_place(self.cur_mod, span.shift(self.offset)))
            .flatten();
        let span = self.span(span);
        let diag = match inside {
            Some((module, _, item)) => cpl_diag::msgs::check::NOT_A_DOMAIN_IN_LIBRARY
                .error(span, &[&item, &module, &param, &what]),
            None => cpl_diag::msgs::check::NOT_A_DOMAIN.error(span, &[&param, &what]),
        };
        self.diags.push(diag);
        self.help(format!(
            "declare a domain, as in `domain {param} = 1..=n`, and pass its name"
        ));
        None
    }

    /// `to(x)` for an `x: from` no conversion takes. For `Int(a)` with
    /// `a: Mod<P>`, a struct whose one field already has the target type,
    /// the help names the field, `a.v`.
    pub(super) fn bad_conversion(&mut self, to: &str, from: &Ty, span: Span) {
        self.report(
            span,
            msgs::ir::TYPE_MISMATCH_A_CAN_T_BE,
            &[&self.show(from), &to],
        );
        let Ty::Struct(s) = from else { return };
        let into = matches!(to, "Int" | "F64").then(|| {
            format!(
                "or give it a conversion: `impl Into<{to}> for {}: fn into(self) -> {to}`",
                self.structs[*s].def.name
            )
        });
        let [(field, ty)] = &self.structs[*s].def.fields[..] else {
            if let Some(into) = into {
                self.help(into.replacen("or give", "give", 1));
            }
            return;
        };
        let fits = matches!(
            (to, ty),
            ("Int", Ty::Int)
                | ("I128", Ty::I128)
                | ("U64", Ty::U64)
                | ("F64", Ty::Float)
                | ("Char", Ty::Char)
                | ("Bool", Ty::Bool)
        );
        if !fits {
            return;
        }
        let call = self.source(span);
        let arg = call
            .strip_prefix(to)
            .and_then(|c| c.strip_prefix('('))
            .and_then(|c| c.strip_suffix(')'))
            .map(str::trim)
            .filter(|a| a.chars().all(|c| c.is_alphanumeric() || "_.".contains(c)));
        let take = match arg {
            Some(a) => format!("`{a}.{field}`"),
            None => format!("`.{field}` on it"),
        };
        self.help(format!(
            "write {take}: a `{}` keeps its value in the field `{field}`, so naming the field is the conversion",
            self.show(from)
        ));
    }

    /// `if let`/`!` on a value that is not optional. For `if let r = i - 1`
    /// with `i: D` the offset is an `Int`, and the fix is `D.at(i - 1)`,
    /// the `?D` that is `None` off the end.
    pub(super) fn not_optional(&mut self, what: &str, value: &Expr, span: Span) {
        self.report(
            span,
            msgs::ir::TYPE_MISMATCH_TAKES_THE_VALUE_OUT_2,
            &[&what, &self.show(&value.ty)],
        );
        let ExprKind::Binary(_, lhs, _) = &value.kind else {
            return;
        };
        let ExprKind::Cast(Cast::Label(d), _) = lhs.kind else {
            return;
        };
        if self.cur_mod != 0 {
            return;
        }
        let domain = self.show(&Ty::Pos(d));
        let at = format!("{domain}.at({})", self.source(span));
        self.help(format!(
            "write `{at}`: a position plus an `Int` is an `Int`, and `{domain}.at` gives back a `?{domain}` that is `None` outside `{domain}`"
        ));
        let edit = (self.span(span), at.clone());
        if let Some(last) = self.diags.last_mut() {
            last.fixes.push(cpl_diag::Fix {
                title: format!("write `{at}`"),
                edits: vec![edit],
            });
        }
    }

    /// Range ends no range takes. `Len.first..=Len.last` has `?Len` ends:
    /// the fix is `Len` itself, or `!` on each end. Two ends of one type
    /// are not a mismatch, so they never read "expected `T`, found `T`".
    pub(super) fn bad_range_ends(
        &mut self,
        (lo, hi): (Option<&ast::Expr>, Option<&ast::Expr>),
        (l, r): (&Ty, &Ty),
        inclusive: bool,
        span: Span,
    ) {
        if l != r {
            // One end is the other's type made optional: blame that end.
            let (at, want, found) = match (l, r, lo, hi) {
                (_, Ty::Opt(inner), _, Some(hi)) if **inner == *l => (hi.span, l, r),
                (Ty::Opt(inner), _, Some(lo), _) if **inner == *r => (lo.span, r, l),
                _ => (span, l, r),
            };
            return self.mismatch(at, want, found);
        }
        let shown = self.show(l);
        let Ty::Opt(inner) = l else {
            let message = cpl_diag::msgs::ir::RANGE_S_ENDS_ARE.at(&[&shown]);
            return self.report_said(span, message);
        };
        self.report(span, msgs::ir::TYPE_MISMATCH_A_RANGE_S_ENDS, &[&shown]);
        let (Some(lo), Some(hi)) = (lo, hi) else {
            return;
        };
        let (lo_text, hi_text) = (self.source(lo.span), self.source(hi.span));
        let whole = match &**inner {
            Ty::Pos(d) if inclusive => {
                let d = self.show(&Ty::Pos(*d));
                (lo_text == format!("{d}.first") && hi_text == format!("{d}.last")).then_some(d)
            }
            _ => None,
        };
        let (title, help, edits) = match whole {
            Some(d) => (
                format!("write `{d}`"),
                format!("all of `{d}` is written `{d}`; `{d}.first` and `{d}.last` are `None` when `{d}` is empty"),
                vec![(self.span(span), d)],
            ),
            None => (
                "write `!` on each end".to_string(),
                format!(
                    "`{lo_text}!{}{hi_text}!` takes the values out, trapping if either is `None`",
                    if inclusive { "..=" } else { ".." }
                ),
                vec![
                    (self.span(Span::new(lo.span.end, lo.span.end)), "!".to_string()),
                    (self.span(Span::new(hi.span.end, hi.span.end)), "!".to_string()),
                ],
            ),
        };
        self.help(help);
        if self.cur_mod == 0 {
            if let Some(last) = self.diags.last_mut() {
                last.fixes.push(cpl_diag::Fix { title, edits });
            }
        }
    }

    /// `type V = 0..n` then `V.first`: a range type has no positions, a
    /// `domain` does. `range` is the alias's right side, absolute in the
    /// user file; the fix rewrites its `type` keyword.
    pub(super) fn range_alias_as_domain(&mut self, name: &str, range: Span) {
        let text = &self.mods[0].text;
        let shown = text
            .get(range.start as usize..range.end as usize)
            .unwrap_or("")
            .to_string();
        let head = text.get(..range.start as usize).unwrap_or("");
        let keyword = head.rfind("type").filter(|&at| {
            let between = head[at + 4..].split_whitespace().collect::<Vec<_>>();
            between == [name, "="]
        });
        let help = format!(
            "`type {name} = {shown}` names a range of integers, with `{name}.MIN`, `{name}.MAX` and `{name}.count`; `domain {name} = {shown}` declares positions, which have `{name}.first`, `{name}.last` and index `[{name}]T`"
        );
        let Some(last) = self.diags.last_mut() else {
            return;
        };
        last.help = Some(help);
        if let Some(at) = keyword {
            let at = at as u32;
            last.fixes.push(cpl_diag::Fix {
                title: format!("declare `domain {name}`"),
                edits: vec![(Span::new(at, at + 4), "domain".into())],
            });
        }
    }

    /// The module, one-based line and innermost declaration holding `at`,
    /// a position in the current library module.
    fn library_place(&self, module: usize, at: Span) -> Option<(String, u32, String)> {
        let analysis = *self.mods.get(module)?;
        let module = &self
            .mods
            .iter()
            .flat_map(|m| m.modules())
            .find(|m| std::ptr::eq(&*m.analysis, analysis))?
            .name;
        let item = analysis
            .items
            .iter()
            .find(|item| item.span.touches(at.start))?;
        let line = analysis.lines.line_of(at.start) + 1;
        if let Some(name) = fn_at(&item.syntax.parsed.stmts, item.offset, at.start, None) {
            return Some((module.clone(), line, name));
        }
        let inside = item
            .syntax
            .decls
            .iter()
            .filter(|d| d.span.shift(item.offset).touches(at.start))
            .min_by_key(|d| (d.kind != cpl_check::DeclKind::Fn, d.span.end - d.span.start))
            .or(item.syntax.decls.first())?;
        Some((module.clone(), line, inside.name.clone()))
    }
}

/// The fn whose body holds `at`, named as a call stack names it (`Stepped.at`).
fn fn_at(
    stmts: &[cpl_syntax::ast::Stmt],
    offset: u32,
    at: u32,
    owner: Option<&str>,
) -> Option<String> {
    use cpl_syntax::ast::StmtKind;
    let stmt = stmts.iter().find(|s| s.span.shift(offset).touches(at))?;
    match &stmt.kind {
        StmtKind::Fn(f) => Some(match owner {
            Some(owner) => format!("{owner}.{}", f.name.text),
            None => f.name.text.clone(),
        }),
        StmtKind::Struct(d) => fn_at(&d.methods, offset, at, Some(&d.name.text)),
        StmtKind::Enum(d) => fn_at(&d.methods, offset, at, Some(&d.name.text)),
        StmtKind::Impl { body, .. } => fn_at(&body.stmts, offset, at, None),
        _ => None,
    }
}

/// Where library code was asked for: the user's expression, the library
/// expression being lowered, and the library calls in between, outermost
/// first, which an error inside the library lists as notes.
#[derive(Clone, Default)]
pub(super) struct Asked {
    pub(super) at: Span,
    lib: Span,
    via: Vec<(usize, Span)>,
}

impl Asked {
    /// Enters expression `span` of module `module`; `leave` takes the result.
    pub(super) fn enter(&mut self, span: Span, module: usize) -> (Span, Span) {
        let outer = (self.at, self.lib);
        if module == 0 {
            self.at = span;
        } else {
            self.lib = span;
        }
        outer
    }

    pub(super) fn leave(&mut self, (at, lib): (Span, Span)) {
        self.at = at;
        self.lib = lib;
    }

    /// What asks for a fn called from module `module` here.
    pub(super) fn callee(&self, module: usize) -> Asked {
        let mut via = Vec::new();
        if module != 0 {
            via = self.via.clone();
            via.push((module, self.lib));
        }
        Asked {
            at: self.at,
            lib: Span::default(),
            via,
        }
    }
}

impl<'a> Lowerer<'a> {
    /// `assert i in s.keys` is the `bounds` check, not an `asserts` one,
    /// and traps as an index does (R09.3.10). The report of an `assert` whose whole condition is `i in s.keys`:
    /// "key 5 is not one of `a.keys`". `None` for any other condition.
    pub(super) fn key_test_report(&mut self, cond: &ast::Expr) -> Option<Expr> {
        let ast::ExprKind::Compare { first, rest } = &cond.kind else {
            return None;
        };
        let [(ast::CmpOp::In, keys)] = rest.as_slice() else {
            return None;
        };
        if !matches!(&keys.kind, ast::ExprKind::Field { name, .. } if name.text == "keys") {
            return None;
        }
        let key = self.expr(first, None);
        let tail = format!(" is not one of `{}`", self.source(keys.span));
        let pieces = vec![
            FmtPiece::Text(b"key ".to_vec()),
            FmtPiece::Value(FmtArg {
                value: key,
                digits: crate::FLOAT_DIGITS,
            }),
            FmtPiece::Text(tail.into_bytes()),
        ];
        Some(self.mk(ExprKind::Format(pieces), Ty::Str, cond.span))
    }
}

/// The plain values a failed condition compared, as written: names and
/// field chains (`i`, `_cap`, `other.len`), also inside the arithmetic of
/// an operand (`x < 1 << _bits` shows `x` and `_bits`), never a call or
/// an index, so showing them can't trap or run anything.
fn compared_values<'e>(e: &'e ast::Expr, out: &mut Vec<&'e ast::Expr>) {
    match &e.kind {
        ast::ExprKind::Compare { first, rest } => {
            for operand in std::iter::once(&**first).chain(rest.iter().map(|(_, x)| x)) {
                operand_leaves(operand, out);
            }
        }
        ast::ExprKind::Binary {
            op: ast::BinOp::And | ast::BinOp::Or,
            lhs,
            rhs,
            ..
        } => {
            compared_values(lhs, out);
            compared_values(rhs, out);
        }
        ast::ExprKind::Unary {
            op: ast::UnOp::Not,
            expr,
        }
        | ast::ExprKind::Paren(expr) => compared_values(expr, out),
        _ => {}
    }
}

fn operand_leaves<'e>(e: &'e ast::Expr, out: &mut Vec<&'e ast::Expr>) {
    match &e.kind {
        _ if plain_value(e) => out.push(e),
        ast::ExprKind::Binary { lhs, rhs, op, .. }
            if !matches!(op, ast::BinOp::And | ast::BinOp::Or | ast::BinOp::Coalesce) =>
        {
            operand_leaves(lhs, out);
            operand_leaves(rhs, out);
        }
        ast::ExprKind::Unary { expr, .. } | ast::ExprKind::Paren(expr) => operand_leaves(expr, out),
        _ => {}
    }
}

/// `i`, `_cap`, `self.lo`, `other.len`: a name or a field chain on one,
/// not a constant (`Int.MAX`, `MOD`).
fn plain_value(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(n) => n.starts_with(|c: char| c.is_ascii_lowercase() || c == '_'),
        ast::ExprKind::Field { base, name } => {
            !name.text.is_empty()
                && (matches!(base.kind, ast::ExprKind::SelfValue) || plain_value(base))
        }
        _ => false,
    }
}

impl<'a> Lowerer<'a> {
    /// What a failed `assert` shows after its message: each plain value
    /// the condition compared, `i = 7, _cap = 4` (errors/R11). `None` when
    /// it compared none.
    pub(super) fn assert_values(&mut self, cond: &ast::Expr) -> Option<Expr> {
        let mut leaves = Vec::new();
        compared_values(cond, &mut leaves);
        let mut seen = std::collections::HashSet::new();
        let mut pieces = Vec::new();
        for leaf in leaves {
            let text = self.source(leaf.span);
            if !seen.insert(text.clone()) {
                continue;
            }
            let value = self.expr(leaf, None);
            let Some(shown) = self.shown(value, leaf.span) else {
                continue;
            };
            let sep = if pieces.is_empty() { "" } else { ", " };
            pieces.push(FmtPiece::Text(format!("{sep}{text} = ").into_bytes()));
            pieces.extend(shown);
        }
        (!pieces.is_empty()).then(|| self.mk(ExprKind::Format(pieces), Ty::Str, cond.span))
    }

    /// How a trap message shows `value`: a range as `lo..hi`, an optional as
    /// `None` or `Some(x)` (`Some(..)` when `x` has no text), a `Bool` as
    /// `true` or `false`, and anything that prints as it prints; `None` when
    /// the type has no text.
    fn shown(&mut self, value: Expr, span: Span) -> Option<Vec<FmtPiece>> {
        let text = |s: &str| FmtPiece::Text(s.as_bytes().to_vec());
        let ty = value.ty.clone();
        match &ty {
            Ty::Bool => {
                let word = |lw: &mut Self, w: &str| lw.mk(ExprKind::Str(w.into()), Ty::Str, span);
                let (yes, no) = (word(self, "true"), word(self, "false"));
                let chosen = ExprKind::If(Box::new(value), Box::new(yes), Box::new(no));
                let chosen = self.mk(chosen, Ty::Str, span);
                Some(vec![self.shown_value(chosen)])
            }
            Ty::Opt(inner) => {
                let inside = ExprKind::Unwrap(Box::new(value.clone()), false);
                let inside = self.mk(inside, (**inner).clone(), span);
                let mut some = vec![text("Some(")];
                some.extend(self.shown(inside, span).unwrap_or_else(|| vec![text("..")]));
                some.push(text(")"));
                let some = self.mk(ExprKind::Format(some), Ty::Str, span);
                let none = self.mk(ExprKind::Str(b"None".to_vec()), Ty::Str, span);
                let test = self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span);
                let chosen = ExprKind::If(Box::new(test), Box::new(some), Box::new(none));
                let chosen = self.mk(chosen, Ty::Str, span);
                Some(vec![self.shown_value(chosen)])
            }
            Ty::Range(inner) => {
                let end = |lw: &mut Self, b: Builtin| {
                    let res = match &**inner {
                        Ty::I128 | Ty::Float => (**inner).clone(),
                        _ => Ty::Int,
                    };
                    let at = lw.mk(ExprKind::Builtin(b, vec![value.clone()]), res, span);
                    match &**inner {
                        Ty::Pos(d) | Ty::Bound(d) => lw.cast(Cast::Trust, at, Ty::Bound(*d)),
                        _ => at,
                    }
                };
                let (lo, hi) = (end(self, Builtin::RangeLo), end(self, Builtin::RangeHi));
                let mut out = self.shown(lo, span)?;
                out.push(text(".."));
                out.extend(self.shown(hi, span)?);
                Some(out)
            }
            _ if self.prints(&ty, &mut Vec::new()) => {
                let arg = self.fmt_arg(value, crate::FLOAT_DIGITS, span)?;
                Some(vec![FmtPiece::Value(arg)])
            }
            _ => None,
        }
    }

    fn shown_value(&self, value: Expr) -> FmtPiece {
        FmtPiece::Value(FmtArg {
            value,
            digits: crate::FLOAT_DIGITS,
        })
    }

    /// Whether `ty` has a text a format string can show: as `print` decides,
    /// a struct shows through its `impl Show`, else field by field.
    fn prints(&self, ty: &Ty, seen: &mut Vec<StructId>) -> bool {
        match ty {
            Ty::Int | Ty::U64 | Ty::I128 | Ty::Float | Ty::Char | Ty::Str => true,
            Ty::Pos(d) => !self.product_domains.contains_key(d),
            Ty::Bound(_) => true,
            Ty::List(e) | Ty::Set(e) => self.prints(e, seen),
            Ty::Array { elem, .. } => self.prints(elem, seen),
            Ty::Tuple(parts) => parts.iter().all(|p| self.prints(p, seen)),
            Ty::Struct(id) if self.structs[*id].def.show_fn.is_some() || seen.contains(id) => true,
            Ty::Struct(id) => {
                seen.push(*id);
                let fields = self.shown_fields(*id);
                fields.iter().all(|(_, t)| self.prints(t, seen))
            }
            _ => false,
        }
    }

    /// `message`, then the values the condition compared: `"needs k >= 0 (k = -1)"`.
    pub(super) fn with_values(
        &mut self,
        message: Option<Expr>,
        values: Option<Expr>,
    ) -> Option<Expr> {
        let Some(values) = values else {
            return message;
        };
        let Some(message) = message else {
            return Some(values);
        };
        let span = message.span;
        let arg = |value| {
            FmtPiece::Value(FmtArg {
                value,
                digits: crate::FLOAT_DIGITS,
            })
        };
        let pieces = vec![
            arg(message),
            FmtPiece::Text(b" (".to_vec()),
            arg(values),
            FmtPiece::Text(b")".to_vec()),
        ];
        Some(Expr {
            kind: ExprKind::Format(pieces),
            ty: Ty::Str,
            span,
        })
    }
}
