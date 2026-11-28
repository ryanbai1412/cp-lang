//! Patterns: the test a value meets and the names it binds (R01.7), and whether a list of them
//! covers a type (R02.7.4).
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::grammar::Decorator;
use cpl_next_syntax::literal::{char_value, int_value};
use cpl_next_syntax::{SyntaxToken, TokenKind};

/// A pattern as far as covering goes: what it matches of a value's shape.
#[derive(Clone, Debug)]
pub(super) enum Space {
    /// Anything: `_`, a name.
    Any,
    /// Constructor `k` of the type (a variant, `None`/`some`, `false`/`true`, the one of a
    /// tuple or struct), with the spaces of its parts.
    Ctor(usize, Vec<Space>),
    /// A literal of a type with no end of values; `None` is one equal to no other.
    Lit(Option<i128>),
    Or(Vec<Space>),
}

/// What a path pattern is, read as the old grammar did (R01.7): a lowercase word binds, `None`
/// is the empty optional, and anything else names a constructor.
enum PathForm {
    /// `x`, `var x`, `self`.
    Bind(SyntaxToken),
    /// `None`.
    None,
    /// `V`, `E.V(p, q)`, `some(p)`: the path's words and the fields' patterns.
    Ctor(Vec<SyntaxToken>, Vec<next::Pattern>),
}

/// One part of a tuple, record or variant against its field: a pattern, or a record field's
/// name alone (`u:`), which binds `u`.
#[derive(Clone, Copy)]
enum Part<'p> {
    Pat(&'p next::Pattern),
    Name(&'p SyntaxToken),
}

impl<'p> Part<'p> {
    fn of(n: &'p SyntaxToken, p: Option<&'p next::Pattern>) -> Self {
        p.map_or(Part::Name(n), Part::Pat)
    }
}

impl Cx<'_> {
    /// `a and b`, either one a test that always holds when absent.
    pub(super) fn both(a: Option<TExpr>, b: Option<TExpr>) -> Option<TExpr> {
        match (a, b) {
            (Some(a), Some(b)) => {
                let span = a.span;
                let no = TExpr {
                    kind: TK::Bool(false),
                    ty: Ty::Bool,
                    span,
                };
                Some(TExpr {
                    kind: TK::If(Box::new(a), Box::new(b), Box::new(no)),
                    ty: Ty::Bool,
                    span,
                })
            }
            (a, b) => a.or(b),
        }
    }
    /// A new name for `ty` the pattern at `at` binds, fixed unless it is a `var`.
    fn bind_as(&mut self, name: &str, ty: Ty, mutable: bool, at: Span) -> VarRef {
        let var = self.bind(name, ty);
        let let_at = (!mutable).then(|| self.let_keyword(at)).flatten();
        let l = match var {
            VarRef::Local(i) => &mut self.b.locals[i],
            VarRef::Global(i) => &mut self.globals[i],
        };
        (l.mutable, l.fixed, l.let_at) = (mutable, !mutable, let_at);
        var
    }
    pub(super) fn var_name(&self, v: VarRef) -> &str {
        match v {
            VarRef::Local(i) => &self.b.locals[i].name,
            VarRef::Global(i) => &self.globals[i].name,
        }
    }
    /// The enum `t` is, with its type arguments.
    fn enum_of(&self, t: &Ty) -> Option<(StructId, Vec<Ty>)> {
        match self.shallow(t) {
            Ty::Struct(e, ts, _) if !self.items.s(e).variants.is_empty() => Some((e, ts)),
            _ => None,
        }
    }
    /// The types of the fields of struct `t`, by their position.
    pub(super) fn field_tys(&self, t: &Ty) -> Option<(StructId, Vec<Ty>)> {
        let Ty::Struct(s, ts, _) = self.shallow(t) else {
            return None;
        };
        let sub = Subst {
            tys: &ts,
            sets: &[],
        };
        let decl = self.items.s(s);
        let fields = decl.fields.iter().map(|(_, t)| sub.ty(t)).collect();
        (decl.variants.is_empty()).then_some((s, fields))
    }
    /// The payload fields of variant `i` of the enum `e` at `targs`.
    fn variant_tys(&self, e: StructId, targs: &[Ty], i: usize) -> (StructId, Vec<Ty>) {
        let p = self.items.s(e).variants[i].1;
        let sub = Subst {
            tys: targs,
            sets: &[],
        };
        let fields = self
            .items
            .s(p)
            .fields
            .iter()
            .map(|(_, t)| sub.ty(t))
            .collect();
        (p, fields)
    }
    /// What the path pattern `p` is; `None` when it is broken.
    fn path_form(p: &next::PathPat) -> Option<PathForm> {
        let segs: Vec<SyntaxToken> = p.segments().collect();
        let args: Option<Vec<next::Pattern>> = p.args().map(|a| a.patterns().collect());
        Some(match (&segs[..], args) {
            ([], _) => return None,
            ([one], None) if super::expr::none_word(one) => PathForm::None,
            ([one], None)
                if one.kind() == TokenKind::SelfKw
                    || !one.text().starts_with(|c: char| c.is_ascii_uppercase()) =>
            {
                PathForm::Bind(one.clone())
            }
            (_, args) => PathForm::Ctor(segs, args.unwrap_or_default()),
        })
    }
    /// Whether `pat` is written against an optional itself (`Some(p)`, `None`), not its value.
    pub(super) fn optional_pattern(&self, pat: &next::Pattern) -> bool {
        match pat {
            next::Pattern::Wildcard(_) => true,
            next::Pattern::Paren(p) => p.pattern().is_none_or(|p| self.optional_pattern(&p)),
            next::Pattern::Literal(l) => l.value().is_none(),
            next::Pattern::Path(p) => match Self::path_form(p) {
                None | Some(PathForm::None) => true,
                Some(PathForm::Ctor(path, _)) => {
                    path.len() == 1 && self.present_pattern(path[0].text())
                }
                Some(PathForm::Bind(_)) => false,
            },
            next::Pattern::Or(o) => o.alternatives().any(|p| self.optional_pattern(&p)),
            next::Pattern::Tuple(_) | next::Pattern::Record(_) => false,
        }
    }
    /// `n`, bound to no type's value.
    fn bind_error(&mut self, n: &str, mutable: bool, span: Span, binds: &mut Vec<TStmt>) {
        let l = self.bind_as(n, Ty::Error, mutable, span);
        let e = TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        };
        binds.push(TStmt::Let(l, e));
    }
    /// The names `pat` binds, of no type, for a value the checker could not type.
    pub(super) fn bind_unknown(
        &mut self,
        pat: &next::Pattern,
        mutable: bool,
        binds: &mut Vec<TStmt>,
    ) {
        let span = self.at(pat);
        match pat {
            next::Pattern::Path(p) => match Self::path_form(p) {
                Some(PathForm::Bind(n)) => {
                    self.bind_error(n.text(), mutable || p.is_var(), span, binds)
                }
                Some(PathForm::Ctor(_, args)) => {
                    (args.iter()).for_each(|a| self.bind_unknown(a, mutable, binds))
                }
                None | Some(PathForm::None) => {}
            },
            next::Pattern::Paren(p) => {
                if let Some(p) = p.pattern() {
                    self.bind_unknown(&p, mutable, binds)
                }
            }
            next::Pattern::Tuple(t) => {
                (t.patterns()).for_each(|p| self.bind_unknown(&p, mutable, binds))
            }
            next::Pattern::Record(r) => {
                for f in r.fields() {
                    match (f.pattern(), f.name()) {
                        (Some(p), _) => self.bind_unknown(&p, mutable, binds),
                        (None, Some(n)) => self.bind_error(n.text(), mutable, n.span(), binds),
                        (None, None) => {}
                    }
                }
            }
            next::Pattern::Or(o) => {
                if let Some(p) = o.alternatives().next() {
                    self.bind_unknown(&p, mutable, binds)
                }
            }
            next::Pattern::Wildcard(_) | next::Pattern::Literal(_) => {}
        }
    }
    /// Whether `head`, where the body is, names an `@pattern` fn: `Some`.
    fn present_pattern(&self, head: &str) -> bool {
        (self.fn_named(self.b.module, head))
            .is_some_and(|f| self.rows.marked(f, Decorator::Pattern))
    }
    /// A tuple pattern of `n` at `at` that can't open `t`, what `callee` gives (its call ends
    /// at `end`): `callee`'s `@on_destructure` says why, with its fix (R02.5.5). Whether it did.
    pub(super) fn destructure_said(
        &mut self,
        callee: FnId,
        n: usize,
        t: &Ty,
        at: Span,
        end: u32,
    ) -> bool {
        let Some(said) = self.items.f(callee).on_destructure.clone() else {
            return false;
        };
        let opens = match self.field_tys(t) {
            Some((s, _)) => self.items.s(s).record || self.shapes.tuple(n) == Some(s),
            None => true,
        };
        if opens || self.mentions_error(t) {
            return false;
        }
        let text = (said.help.unwrap_or_default()).replace("{n}", &n.to_string());
        let code = said.code.unwrap_or(cpl_diag::codes::TYPE_MISMATCH);
        let mut msg = Msg::verdict(code, text);
        if let Some(fix) = said.fix {
            let write = cpl_diag::habits::WRITE.say(&[&fix]);
            msg.help = Some(write.clone());
            msg.fixes.push(cpl_next_ir::msg::Fix {
                title: write,
                edits: vec![(self.b.item.at(Span::new(end, end)), fix)],
            });
        }
        self.err(at, msg);
        true
    }
    pub(super) fn mismatch(&mut self, span: Span, what: &str, t: &Ty) {
        if self.shallow(t) == Ty::Error {
            return;
        }
        let code = cpl_diag::codes::TYPE_MISMATCH;
        let t = self.show(t);
        self.err(
            span,
            Msg::verdict(code, format!("{what} cannot match a `{t}`")),
        );
    }

    /// `v` holds no value: the pattern `None`'s test, on the optional's flag.
    fn absent(v: TExpr, span: Span) -> TExpr {
        let flag = TExpr {
            kind: TK::Field(Box::new(v), 0),
            ty: Ty::Bool,
            span,
        };
        TExpr {
            kind: TK::Not(Box::new(flag)),
            ty: Ty::Bool,
            span,
        }
    }
    /// A `?T` compared with the `None` value: the pattern `None`'s test, so `T` needs no `Eq`
    /// (R02.4.5). `None` when neither side alone is the `None` value.
    pub(super) fn presence(&self, op: CmpOp, a: &TExpr, b: &TExpr, span: Span) -> Option<TExpr> {
        if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            return None;
        }
        // `n == None` with `n` a `let` of the None value tests `n`, as any `?T` would be.
        let x = match (self.is_none_value(a), self.is_none_value(b)) {
            (_, true) => a,
            (true, false) => b,
            _ => return None,
        };
        self.optional_of(&x.ty)?;
        let test = Self::absent(x.clone(), span);
        Some(match op {
            CmpOp::Ne => TExpr {
                kind: TK::Not(Box::new(test)),
                ty: Ty::Bool,
                span,
            },
            _ => test,
        })
    }
    /// Whether `e` is the `None` value: built as it, or a `let` bound to it.
    pub(super) fn is_none_value(&self, e: &TExpr) -> bool {
        match &e.kind {
            TK::Struct(xs) => matches!(
                xs.as_slice(),
                [
                    TExpr {
                        kind: TK::Bool(false),
                        ..
                    },
                    TExpr {
                        kind: TK::Unset,
                        ..
                    }
                ]
            ),
            TK::Var(v) => self.none_lets.contains(v),
            _ => false,
        }
    }
    /// The test that `v` is variant `i` of its enum.
    fn tag_is(v: &TExpr, i: usize, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let tag = mk(TK::Tag(Box::new(v.clone())), Ty::Int);
        let i = mk(TK::Int(i as i64), Ty::Int);
        mk(TK::Cmp(CmpOp::Eq, Box::new(tag), Box::new(i)), Ty::Bool)
    }
    /// Whether `v`'s type is not known, so a pattern against it only binds its names.
    fn untyped(&mut self, v: &TExpr, span: Span) -> bool {
        let t = self.shallow(&v.ty);
        if matches!(t, Ty::Var(_)) {
            self.err(span, Unbuilt::PatternOnUnknownType);
        }
        matches!(t, Ty::Error | Ty::Var(_))
    }
    /// Whether `pat` is one name that binds, as `x` is and `None` or `E.V(..)` are not.
    pub(super) fn name_pattern(pat: &next::Pattern) -> bool {
        matches!(pat, next::Pattern::Path(p) if bind_name(p).is_some())
    }
    /// `pat` against the place `v`: the test that holds when it matches, `None` when it always
    /// does. `binds` gets the names it binds, read once the test holds.
    pub(super) fn refute(
        &mut self,
        pat: &next::Pattern,
        v: &TExpr,
        mutable: bool,
        binds: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        let span = self.at(pat);
        let mk = |kind, ty| TExpr { kind, ty, span };
        if self.untyped(v, span) {
            self.bind_unknown(pat, mutable, binds);
            return None;
        }
        match pat {
            next::Pattern::Wildcard(_) => None,
            next::Pattern::Paren(p) => p.pattern().and_then(|p| self.refute(&p, v, mutable, binds)),
            next::Pattern::Path(p) => {
                let mutable = mutable || p.is_var();
                match Self::path_form(p)? {
                    PathForm::Bind(n) => self.refute_name(n.text(), span, v, mutable, binds),
                    PathForm::None => {
                        if self.optional_of(&v.ty).is_none() {
                            self.mismatch(span, "`None`", &v.ty);
                            return None;
                        }
                        Some(Self::absent(v.clone(), span))
                    }
                    PathForm::Ctor(path, args) => {
                        self.refute_ctor(&path, &args, span, v, mutable, binds)
                    }
                }
            }
            next::Pattern::Literal(l) => {
                let e = l.value()?;
                let t = match self.shallow(&v.ty) {
                    t if t.is_int_range() => Ty::Int,
                    t @ (Ty::Int | Ty::Bool | Ty::Char | Ty::U64 | Ty::F64) => t,
                    Ty::Error => return None,
                    // Any other literal, such as a string, matches what its type's `Eq` says
                    // is equal (R01.7).
                    t @ Ty::Struct(..) => {
                        let lit = self.expr(&e, Some(&t));
                        let lit = self.coerce(lit, &t);
                        return Some(self.op_call(CmpOp::Eq, v.clone(), lit, span));
                    }
                    t => {
                        let t = self.show(&t);
                        self.err(span, Unbuilt::LiteralPattern { ty: t });
                        return None;
                    }
                };
                let here = self.coerce(v.clone(), &t);
                let lit = self.expr(&e, Some(&t));
                let lit = self.coerce(lit, &t);
                Some(mk(
                    TK::Cmp(CmpOp::Eq, Box::new(here), Box::new(lit)),
                    Ty::Bool,
                ))
            }
            next::Pattern::Tuple(tp) => {
                let ps: Vec<next::Pattern> = tp.patterns().collect();
                let tuple = self.shapes.tuple(ps.len());
                match self.field_tys(&v.ty) {
                    Some((s, tys))
                        if Some(s) == tuple
                            || (tys.len() == ps.len() && self.items.s(s).record) =>
                    {
                        self.pat_parts(ps.iter().map(Part::Pat).zip(0..), v, &tys, mutable, binds)
                    }
                    _ => {
                        let what = format!("a tuple pattern of {}", ps.len());
                        self.mismatch(span, &what, &v.ty);
                        None
                    }
                }
            }
            next::Pattern::Record(r) => {
                let Some((s, tys)) = self.field_tys(&v.ty) else {
                    self.mismatch(span, "a record pattern", &v.ty);
                    return None;
                };
                let names: Vec<String> = self.field_names(s);
                let fields: Vec<(SyntaxToken, Option<next::Pattern>)> = (r.fields())
                    .filter_map(|f| Some((f.name()?, f.pattern())))
                    .collect();
                if !r.has_rest() && fields.len() != names.len() {
                    let code = cpl_diag::codes::TYPE_MISMATCH;
                    let have = names.len();
                    let n = fields.len();
                    self.err(span, Msg::verdict(code, format!("this names {n} of the record's {have} fields; write `..` for the rest")));
                }
                let mut at = Vec::new();
                for (n, p) in &fields {
                    match names.iter().position(|m| *m == n.text()) {
                        Some(k) => at.push((Part::of(n, p.as_ref()), k)),
                        None => {
                            let code = cpl_diag::codes::UNKNOWN_FIELD;
                            let t = self.show(&v.ty);
                            self.err(
                                n.span(),
                                Msg::verdict(code, format!("`{t}` has no field `{}`", n.text())),
                            );
                        }
                    }
                }
                self.pat_parts(at.into_iter(), v, &tys, mutable, binds)
            }
            next::Pattern::Or(o) => {
                let alts: Vec<next::Pattern> = o.alternatives().collect();
                self.alternatives(&alts, v, mutable, binds)
            }
        }
    }
    /// The name `n` against `v`: the test that `v` is the variant `n`, if its enum has one,
    /// else `n` bound to `v`.
    fn refute_name(
        &mut self,
        n: &str,
        span: Span,
        v: &TExpr,
        mutable: bool,
        binds: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        if let Some((e, _)) = self.enum_of(&v.ty) {
            if let Some(i) = self.variant_named(e, n) {
                return Some(Self::tag_is(v, i, span));
            }
        }
        let l = self.bind_as(n, v.ty.clone(), mutable, span);
        binds.push(TStmt::Let(l, v.clone()));
        None
    }
    /// `E.V(p, q)`, `some(p)` or the domain pattern `V(j)` against `v`.
    fn refute_ctor(
        &mut self,
        path: &[SyntaxToken],
        args: &[next::Pattern],
        span: Span,
        v: &TExpr,
        mutable: bool,
        binds: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let mut head = path[path.len() - 1].text();
        // `some(p)` on an optional is read as `Some(p)` once its E0503 is said.
        if path.len() == 1
            && !self.present_pattern(head)
            && self.optional_of(&v.ty).is_some()
            && self.old_optional_spelling(head, path[0].span())
        {
            head = "Some";
        }
        if path.len() == 1 && self.present_pattern(head) {
            let Some(inner) = self.optional_of(&v.ty) else {
                self.mismatch(span, &format!("`{head}(..)`"), &v.ty);
                return None;
            };
            let [p] = args else {
                let code = cpl_diag::codes::WRONG_ARG_COUNT;
                self.err(
                    span,
                    Msg::verdict(
                        code,
                        format!("`{head}(..)` takes 1 pattern, found {}", args.len()),
                    ),
                );
                return None;
            };
            let flag = mk(TK::Field(Box::new(v.clone()), 0), Ty::Bool);
            let val = mk(TK::Field(Box::new(v.clone()), 1), inner);
            let rest = self.refute(p, &val, mutable, binds);
            return Self::both(Some(flag), rest);
        }
        // `V(j)` is the pattern form of `V.at(x)`: an `Int` inside `V` (R04.2.14).
        if let (1, Some(dty)) = (path.len(), self.domain_named(head)) {
            let [p] = args else {
                let code = cpl_diag::codes::WRONG_ARG_COUNT;
                self.err(
                    span,
                    Msg::verdict(
                        code,
                        format!("`{head}(..)` takes 1 pattern, found {}", args.len()),
                    ),
                );
                return None;
            };
            let t = self.shallow(&v.ty);
            if t != Ty::Int && !t.is_int_range() {
                self.mismatch(span, &format!("`{head}(..)`"), &v.ty);
                return None;
            }
            let x = self.coerce(v.clone(), &Ty::Int);
            let at = self.domain_at(dty.clone(), x, span);
            let flag = mk(TK::Field(Box::new(at.clone()), 0), Ty::Bool);
            let val = mk(TK::Field(Box::new(at), 1), dty);
            let rest = self.refute(p, &val, mutable, binds);
            return Self::both(Some(flag), rest);
        }
        let Some((e, targs)) = self.enum_of(&v.ty) else {
            self.mismatch(span, &format!("`{head}`"), &v.ty);
            return None;
        };
        let Some(i) = self.variant_named(e, head) else {
            let en = self.items.s(e).name.clone();
            let code = cpl_diag::codes::UNKNOWN_FIELD;
            self.err(
                span,
                Msg::verdict(code, format!("`{en}` has no variant `{head}`")),
            );
            return None;
        };
        let (p, tys) = self.variant_tys(e, &targs, i);
        if !args.is_empty() && args.len() != tys.len() {
            let code = cpl_diag::codes::WRONG_ARG_COUNT;
            let msg = Msg::verdict(
                code,
                format!("`{head}` has {} fields, found {}", tys.len(), args.len()),
            );
            self.err(span, msg);
        }
        let payload = mk(
            TK::Payload(Box::new(v.clone()), i),
            Ty::Struct(p, targs, Vec::new()),
        );
        let rest = self.pat_parts(
            args.iter().map(Part::Pat).zip(0..),
            &payload,
            &tys,
            mutable,
            binds,
        );
        Self::both(Some(Self::tag_is(v, i, span)), rest)
    }
    /// The parts of `v` (its fields `k`, of types `tys`) against their patterns.
    fn pat_parts<'p>(
        &mut self,
        ps: impl Iterator<Item = (Part<'p>, usize)>,
        v: &TExpr,
        tys: &[Ty],
        mutable: bool,
        binds: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        let mut test = None;
        for (p, k) in ps {
            let Some(t) = tys.get(k) else { continue };
            let span = match p {
                Part::Pat(p) => self.at(p),
                Part::Name(n) => n.span(),
            };
            let part = TExpr {
                kind: TK::Field(Box::new(v.clone()), k),
                ty: t.clone(),
                span,
            };
            let t = match p {
                Part::Pat(p) => self.refute(p, &part, mutable, binds),
                Part::Name(n) if self.untyped(&part, span) => {
                    self.bind_error(n.text(), mutable, span, binds);
                    None
                }
                Part::Name(n) => self.refute_name(n.text(), span, &part, mutable, binds),
            };
            test = Self::both(test, t);
        }
        test
    }
    /// `p | q`: the first alternative that matches binds the names, which each one binds alike.
    /// The alternatives are bound last to first, so the names in scope are the first's.
    fn alternatives(
        &mut self,
        alts: &[next::Pattern],
        v: &TExpr,
        mutable: bool,
        binds: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        let mut each: Vec<(Option<TExpr>, Vec<TStmt>)> = Vec::new();
        for a in alts.iter().rev() {
            let mut b = Vec::new();
            let t = self.refute(a, v, mutable, &mut b);
            each.push((t, b));
        }
        each.reverse();
        let names = |cx: &Self, b: &[TStmt]| -> Vec<(String, VarRef)> {
            let mut ns: Vec<_> = (b.iter())
                .filter_map(|s| match s {
                    TStmt::Let(l, _) => Some((cx.var_name(*l).to_string(), *l)),
                    _ => None,
                })
                .collect();
            ns.sort_by(|a, b| a.0.cmp(&b.0));
            ns
        };
        let first = names(self, &each[0].1);
        if first.is_empty() && each.iter().all(|(_, b)| b.is_empty()) {
            return (each.into_iter().map(|(t, _)| t)).try_fold(
                None,
                |acc: Option<TExpr>, t| {
                    let t = t?;
                    Some(Some(match acc {
                        None => t,
                        Some(a) => {
                            let span = a.span;
                            let yes = TExpr {
                                kind: TK::Bool(true),
                                ty: Ty::Bool,
                                span,
                            };
                            TExpr {
                                kind: TK::If(Box::new(a), Box::new(yes), Box::new(t)),
                                ty: Ty::Bool,
                                span,
                            }
                        }
                    }))
                },
            )?;
        }
        let span = v.span;
        let lit = |b| TExpr {
            kind: TK::Bool(b),
            ty: Ty::Bool,
            span,
        };
        let mut chain = lit(false);
        for (t, b) in each.into_iter().rev() {
            let these = names(self, &b);
            if these.iter().map(|n| &n.0).ne(first.iter().map(|n| &n.0)) {
                let code = cpl_diag::codes::TYPE_MISMATCH;
                self.err(
                    span,
                    Msg::verdict(
                        code,
                        "each alternative of a `|` pattern binds the same names",
                    ),
                );
            }
            // Each alternative binds the first's names.
            let b = (b.into_iter())
                .map(|s| match s {
                    TStmt::Let(l, e) => {
                        let name = self.var_name(l);
                        let to = first.iter().find(|n| n.0 == name).map_or(l, |n| n.1);
                        TStmt::Let(to, e)
                    }
                    s => s,
                })
                .collect();
            let then = TExpr {
                kind: TK::Block(b, Some(Box::new(lit(true)))),
                ty: Ty::Bool,
                span,
            };
            chain = match t {
                Some(t) => TExpr {
                    kind: TK::If(Box::new(t), Box::new(then), Box::new(chain)),
                    ty: Ty::Bool,
                    span,
                },
                None => then,
            };
        }
        let _ = binds;
        Some(chain)
    }

    /// What `pat` covers of a value of type `t`.
    /// `domain I` over the positions `t` of a result's `domain C`: `I` is a domain of its own,
    /// its ends read from `C`'s slot as the call returns (R02.9.3). `C` and `I`, if `t` is one.
    pub(super) fn bind_fresh(
        &mut self,
        name: Sym,
        t: &Ty,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> Option<(usize, usize)> {
        let c = self.fresh_slot_of(t)?;
        let shown = self.b.syms.text(self.syms, name).to_string();
        let i = self.fresh_from(&shown, c, span, out);
        self.b.doms.insert(name, Ty::Dom(i));
        Some((c, i))
    }
    /// A domain of its own named `name`, its ends read from the fresh `C`'s slot.
    fn fresh_from(&mut self, name: &str, c: usize, span: Span, out: &mut Vec<TStmt>) -> usize {
        let (lo, len) = (self.domains[c].lo, self.domains[c].len);
        let var = |r| TExpr {
            kind: TK::Var(r),
            ty: Ty::Int,
            span,
        };
        let end = TExpr {
            kind: TK::Bin(BinOp::Add, Box::new(var(lo)), Box::new(var(len))),
            ty: Ty::Int,
            span,
        };
        let info = self.domain_info(name, var(lo), end, (None, None), span, out);
        self.add_domain(info, span)
    }
    /// The fresh domains `t`'s struct holds as its own parameters, each with that parameter's
    /// name: `Pre` of a `Rooted.new` result.
    pub(super) fn fresh_args(&mut self, t: &Ty) -> Vec<(usize, Vec<String>)> {
        let t = self.zonk(t);
        let mut out = Vec::new();
        self.fresh_in(&t, &[], &mut out);
        // A `domain C` binder's domain is bound by its pattern, not owned here.
        let binders: Vec<usize> = match &t {
            Ty::Struct(s, ts, _) if self.items.s(*s).record => {
                ts.iter().filter_map(|a| self.fresh_slot_of(a)).collect()
            }
            _ => Vec::new(),
        };
        out.retain(|(c, _)| !binders.contains(c));
        out
    }
    /// A record's are named through its fields: `s.P` of `(s: S<V, _>, k: Int)`.
    fn fresh_in(&self, t: &Ty, path: &[String], out: &mut Vec<(usize, Vec<String>)>) {
        let Ty::Struct(s, ts, _) = t else {
            return;
        };
        let sd = self.items.s(*s);
        for (k, a) in ts.iter().enumerate() {
            match self.shallow(a) {
                Ty::Dom(c) if !sd.record && self.doms.fresh_dom(c) => {
                    out.push((c, [path, &sd.generics[k..=k]].concat()));
                }
                a if sd.record => {
                    let field = sd
                        .fields
                        .get(k)
                        .map_or(String::new(), |f| self.items.text(f.0).to_string());
                    self.fresh_in(&a, &[path, &[field]].concat(), out);
                }
                _ => {}
            }
        }
    }
    /// `t.Pre`: the domain `t` owns as its result's fresh `Pre`, if `t` is such a binding.
    pub(super) fn owned_domain(&self, base: &next::Expr, name: &SyntaxToken) -> Option<Ty> {
        let next::Expr::Name(b) = base else {
            return None;
        };
        self.owned_at(b.ident()?.text(), &[name.text().to_string()])
    }
    /// `t.Pre` written in a type: the domain the binding `t` owns as its `Pre`.
    pub(super) fn owned_named(&self, n: &str) -> Option<Ty> {
        let mut parts = n.split('.');
        let head = parts.next()?;
        let path: Vec<String> = parts.map(str::to_string).collect();
        self.owned_at(head, &path)
    }
    fn owned_at(&self, binding: &str, path: &[String]) -> Option<Ty> {
        if path.is_empty() {
            return None;
        }
        let (v, _) = self.lookup(binding)?;
        self.b.owned.get(&(v, path.to_vec())).map(|&i| Ty::Dom(i))
    }
    /// `let (tree, ..) = r`: what `r` owns under `tree` the new binding `tree` owns.
    pub(super) fn hand_down(&mut self, whole: Sym, field: Sym, part: Sym) {
        let (whole, part) = (
            self.b.syms.text(self.syms, whole),
            self.b.syms.text(self.syms, part),
        );
        let (Some((from, _)), Some((to, _))) = (self.lookup(whole), self.lookup(part)) else {
            return;
        };
        let field = self.b.syms.text(self.syms, field);
        let moved: Vec<_> = (self.b.owned.iter())
            .filter(|((v, path), _)| *v == from && path.first().is_some_and(|f| f == field))
            .map(|((_, path), &i)| ((to, path[1..].to_vec()), i))
            .collect();
        self.b.owned.extend(moved);
    }
    /// `let t = Rooted.new(..)`: each fresh domain of the result is one of `t`'s own, `t.Pre`,
    /// so two trees' positions never mix (R02.9.1).
    pub(super) fn own_fresh(
        &mut self,
        name: &str,
        var: VarRef,
        fresh: Vec<(usize, Vec<String>)>,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        for (c, p) in fresh {
            let shown = [name].into_iter().chain(p.iter().map(String::as_str));
            let i = self.fresh_from(&shown.collect::<Vec<_>>().join("."), c, span, out);
            self.b.owned.insert((var, p), i);
            let held = self.zonk(&self.var_ty(var));
            let slot = match var {
                VarRef::Local(k) => &mut self.b.locals[k].ty,
                VarRef::Global(k) => &mut self.globals[k].ty,
            };
            *slot = retag(&held, c, i);
        }
    }
    /// The result's `domain C` whose positions `t` is.
    pub(super) fn fresh_slot_of(&self, t: &Ty) -> Option<usize> {
        let r = self.range_struct()?;
        match self.shallow(t) {
            Ty::Struct(s, ts, _) if s == r && ts.len() == 1 => match self.shallow(&ts[0]) {
                Ty::Dom(c) if self.doms.fresh_dom(c) => Some(c),
                _ => None,
            },
            _ => None,
        }
    }
    pub(super) fn space(&self, pat: &next::Pattern, t: &Ty) -> Space {
        match pat {
            next::Pattern::Wildcard(_) => Space::Any,
            next::Pattern::Paren(p) => p.pattern().map_or(Space::Any, |p| self.space(&p, t)),
            next::Pattern::Path(p) => match Self::path_form(p) {
                None => Space::Any,
                Some(PathForm::Bind(n)) => self.name_space(n.text(), t),
                Some(PathForm::None) => Space::Ctor(0, Vec::new()),
                Some(PathForm::Ctor(path, args)) => self.ctor_space(&path, &args, t),
            },
            next::Pattern::Literal(l) => {
                l.value().map_or(Space::Any, |e| self.literal_space(&e, t))
            }
            next::Pattern::Tuple(tp) => {
                let tys = self.field_tys(t).map(|f| f.1).unwrap_or_default();
                let at = |k: usize| tys.get(k).cloned().unwrap_or(Ty::Error);
                Space::Ctor(
                    0,
                    tp.patterns()
                        .enumerate()
                        .map(|(k, p)| self.space(&p, &at(k)))
                        .collect(),
                )
            }
            next::Pattern::Record(r) => match self.field_tys(t) {
                Some((s, tys)) => {
                    let mut parts = vec![Space::Any; tys.len()];
                    for f in r.fields() {
                        let Some(n) = f.name() else { continue };
                        if let Some(k) = self.field_index(s, n.text()) {
                            parts[k] = match f.pattern() {
                                Some(p) => self.space(&p, &tys[k]),
                                None => self.name_space(n.text(), &tys[k]),
                            };
                        }
                    }
                    Space::Ctor(0, parts)
                }
                None => Space::Any,
            },
            next::Pattern::Or(o) => {
                Space::Or(o.alternatives().map(|a| self.space(&a, t)).collect())
            }
        }
    }
    /// What the name `n` covers: the variant it names, else anything.
    fn name_space(&self, n: &str, t: &Ty) -> Space {
        let Some((e, ts)) = self.enum_of(t) else {
            return Space::Any;
        };
        match self.variant_named(e, n) {
            Some(i) => {
                let arity = self.variant_tys(e, &ts, i).1.len();
                Space::Ctor(i, vec![Space::Any; arity])
            }
            None => Space::Any,
        }
    }
    fn ctor_space(&self, path: &[SyntaxToken], args: &[next::Pattern], t: &Ty) -> Space {
        let head = match path[path.len() - 1].text() {
            "some" if path.len() == 1 && !self.present_pattern("some") => "Some",
            h => h,
        };
        if let (Some(inner), [p]) = (self.optional_of(t), args) {
            if path.len() == 1 && self.present_pattern(head) {
                return Space::Ctor(1, vec![self.space(p, &inner)]);
            }
        }
        // A domain pattern tests membership, which no other pattern covers.
        if path.len() == 1 && self.domain_named(head).is_some() {
            return Space::Lit(None);
        }
        let Some((e, ts)) = self.enum_of(t) else {
            return Space::Any;
        };
        match self.variant_named(e, head) {
            Some(i) => {
                let tys = self.variant_tys(e, &ts, i).1;
                let parts = (tys.iter().enumerate())
                    .map(|(k, t)| args.get(k).map_or(Space::Any, |p| self.space(p, t)))
                    .collect();
                Space::Ctor(i, parts)
            }
            None => Space::Any,
        }
    }
    /// What a literal pattern covers: `false`/`true` of a `Bool`, else the one value it is.
    fn literal_space(&self, e: &next::Expr, t: &Ty) -> Space {
        let token = |e: &next::Expr| match e {
            next::Expr::Literal(l) => l.token(),
            _ => None,
        };
        if let next::Expr::Prefix(p) = e {
            let neg = p.op().is_some_and(|o| o.kind() == TokenKind::Minus);
            return match p.operand().as_ref().and_then(token) {
                Some(n) if neg && n.kind() == TokenKind::Int => {
                    Space::Lit(int_value(n.text()).map(|i| -i))
                }
                _ => Space::Lit(None),
            };
        }
        let Some(tok) = token(e) else {
            return Space::Lit(None);
        };
        match (tok.kind(), self.shallow(t)) {
            (TokenKind::True, Ty::Bool) => Space::Ctor(1, Vec::new()),
            (TokenKind::False, Ty::Bool) => Space::Ctor(0, Vec::new()),
            (TokenKind::Int, _) => Space::Lit(int_value(tok.text())),
            (TokenKind::Char, _) => {
                Space::Lit(char_value(tok.text()).map(|c| i128::from(u32::from(c))))
            }
            _ => Space::Lit(None),
        }
    }
    /// The constructors of `t` with the types of their parts, when it has a closed list of them.
    fn ctors(&self, t: &Ty) -> Option<Vec<(String, Vec<Ty>)>> {
        if let Some(inner) = self.optional_of(t) {
            return Some(vec![
                ("None".into(), Vec::new()),
                ("Some".into(), vec![inner]),
            ]);
        }
        if let Some((e, ts)) = self.enum_of(t) {
            let vs = &self.items.s(e).variants;
            return Some(
                (0..vs.len())
                    .map(|i| {
                        (
                            self.items.text(vs[i].0).to_string(),
                            self.variant_tys(e, &ts, i).1,
                        )
                    })
                    .collect(),
            );
        }
        match self.shallow(t) {
            Ty::Bool => Some(vec![
                ("false".into(), Vec::new()),
                ("true".into(), Vec::new()),
            ]),
            _ => self.field_tys(t).map(|(_, tys)| vec![(String::new(), tys)]),
        }
    }
    /// A value of the types `tys` that `q` matches and no row does, if there is one.
    pub(super) fn useful(
        &self,
        rows: &[Vec<Space>],
        q: &[Space],
        tys: &[Ty],
    ) -> Option<Vec<Space>> {
        let Some((head, rest)) = q.split_first() else {
            return rows.is_empty().then(Vec::new);
        };
        // A row headed by `p | q` is a row per alternative.
        let mut flat: Vec<Vec<Space>> = Vec::new();
        for r in rows {
            match &r[0] {
                Space::Or(alts) => flat.extend(
                    alts.iter()
                        .map(|a| [vec![a.clone()], r[1..].to_vec()].concat()),
                ),
                _ => flat.push(r.clone()),
            }
        }
        let rest_tys = &tys[1..];
        let spec = |c: usize, arity: usize| -> Vec<Vec<Space>> {
            (flat.iter())
                .filter_map(|r| match &r[0] {
                    Space::Ctor(k, ps) if *k == c => Some([ps.clone(), r[1..].to_vec()].concat()),
                    Space::Any => Some([vec![Space::Any; arity], r[1..].to_vec()].concat()),
                    _ => None,
                })
                .collect()
        };
        let ctors = self.ctors(&tys[0]);
        match head {
            Space::Or(alts) => alts
                .iter()
                .find_map(|a| self.useful(rows, &[vec![a.clone()], rest.to_vec()].concat(), tys)),
            Space::Ctor(c, ps) => {
                let field_tys = (ctors.as_ref().and_then(|cs| cs.get(*c)))
                    .map_or_else(|| vec![Ty::Error; ps.len()], |c| c.1.clone());
                let w = self.useful(
                    &spec(*c, ps.len()),
                    &[ps.clone(), rest.to_vec()].concat(),
                    &[field_tys, rest_tys.to_vec()].concat(),
                )?;
                let (inner, after) = w.split_at(ps.len());
                Some([vec![Space::Ctor(*c, inner.to_vec())], after.to_vec()].concat())
            }
            Space::Lit(k) => {
                let same: Vec<Vec<Space>> = (flat.iter())
                    .filter(|r| match &r[0] {
                        Space::Any => true,
                        Space::Lit(j) => k.is_some() && j == k,
                        _ => false,
                    })
                    .map(|r| r[1..].to_vec())
                    .collect();
                let w = self.useful(&same, rest, rest_tys)?;
                Some([vec![head.clone()], w].concat())
            }
            Space::Any => {
                let seen = |c: usize| {
                    flat.iter()
                        .any(|r| matches!(r[0], Space::Ctor(k, _) if k == c))
                };
                if let Some(cs) = ctors.as_ref().filter(|cs| (0..cs.len()).all(seen)) {
                    return cs.iter().enumerate().find_map(|(c, (_, ftys))| {
                        let q = [vec![Space::Any; ftys.len()], rest.to_vec()].concat();
                        let w = self.useful(
                            &spec(c, ftys.len()),
                            &q,
                            &[ftys.clone(), rest_tys.to_vec()].concat(),
                        )?;
                        let (inner, after) = w.split_at(ftys.len());
                        Some([vec![Space::Ctor(c, inner.to_vec())], after.to_vec()].concat())
                    });
                }
                let default: Vec<Vec<Space>> = (flat.iter())
                    .filter(|r| matches!(r[0], Space::Any))
                    .map(|r| r[1..].to_vec())
                    .collect();
                let w = self.useful(&default, rest, rest_tys)?;
                let missing = (ctors.as_ref()).and_then(|cs| {
                    (0..cs.len())
                        .find(|c| !seen(*c))
                        .map(|c| Space::Ctor(c, vec![Space::Any; cs[c].1.len()]))
                });
                Some([vec![missing.unwrap_or(Space::Any)], w].concat())
            }
        }
    }
    /// A value `w` of type `t`, as a pattern.
    pub(super) fn show_space(&self, w: &Space, t: &Ty) -> String {
        match w {
            Space::Ctor(c, ps) => {
                let cs = self.ctors(t).unwrap_or_default();
                let Some((name, tys)) = cs.get(*c) else {
                    return "_".into();
                };
                let parts: Vec<String> = ps
                    .iter()
                    .zip(tys)
                    .map(|(p, t)| self.show_space(p, t))
                    .collect();
                match (name.as_str(), parts.is_empty()) {
                    (n, true) if !n.is_empty() => n.to_string(),
                    (n, _) => format!("{n}({})", parts.join(", ")),
                }
            }
            _ => "_".into(),
        }
    }
}

/// `t` names the domain `c` somewhere.
pub(super) fn holds_dom(t: &Ty, c: usize) -> bool {
    match t {
        Ty::Dom(i) => *i == c,
        Ty::Raw(e) | Ty::Ref(e) | Ty::Lent(e) | Ty::RangeTo(e, ..) => holds_dom(e, c),
        Ty::Struct(_, ts, _) => ts.iter().any(|t| holds_dom(t, c)),
        Ty::Fn(ps, r, _) => ps.iter().any(|t| holds_dom(t, c)) || holds_dom(r, c),
        _ => false,
    }
}

/// `t` with domain `from` named `to`: what a caller's `domain I` makes of a result's `[C]T`.
pub(super) fn retag(t: &Ty, from: usize, to: usize) -> Ty {
    let r = |t: &Ty| retag(t, from, to);
    match t {
        Ty::Dom(i) if *i == from => Ty::Dom(to),
        Ty::Raw(e) => Ty::Raw(Box::new(r(e))),
        Ty::RangeTo(e, m, lo, k) => Ty::RangeTo(Box::new(r(e)), *m, *lo, *k),
        Ty::Mut(e) => Ty::Mut(Box::new(r(e))),
        Ty::Ref(e) => Ty::Ref(Box::new(r(e))),
        Ty::Lent(e) => Ty::Lent(Box::new(r(e))),
        Ty::Struct(d, ts, ss) => Ty::Struct(*d, ts.iter().map(r).collect(), ss.clone()),
        Ty::Fn(ps, ret, s) => Ty::Fn(ps.iter().map(r).collect(), Box::new(r(ret)), s.clone()),
        other => other.clone(),
    }
}

/// The name a path pattern binds, as `x`, `var x` and `self` do and `None` or `E.V(..)` do not.
pub(super) fn bind_name(p: &next::PathPat) -> Option<SyntaxToken> {
    match Cx::path_form(p) {
        Some(PathForm::Bind(n)) => Some(n),
        _ => None,
    }
}
