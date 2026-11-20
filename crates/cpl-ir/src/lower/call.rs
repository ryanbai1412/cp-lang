//! Calls, methods and members: the user's fns and the ones the language
//! provides.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reduce {
    Sum,
    Count,
    All,
    Any,
    Min,
    Max,
}

impl<'a> Lowerer<'a> {
    /// The symbol `e` names, if it is a bare name no local hides.
    fn sym(&self, e: &ast::Expr) -> Option<Sym> {
        match &e.kind {
            ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                self.syms.get(name).copied()
            }
            _ => None,
        }
    }

    pub(super) fn call(
        &mut self,
        whole: &ast::Expr,
        callee: &ast::Expr,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        match &callee.kind {
            ast::ExprKind::Paren(inner) => self.call(whole, inner, args, hint),
            ast::ExprKind::Name(name) => {
                if self.lookup(name).is_some() {
                    self.error(
                        codes::NOT_CALLABLE,
                        callee.span,
                        format!("`{name}` is a variable, not a fn"),
                    );
                    return self.err_expr(span);
                }
                match self.syms.get(name).copied() {
                    Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
                    Some(Sym::Struct(id)) => self.construct(id, args, span),
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    Some(Sym::Domain(id)) => {
                        let Some(value) = self.one_arg(name, args, &Ty::Int, span) else {
                            return self.err_expr(span);
                        };
                        self.mk(
                            ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                            Ty::Pos(id),
                            span,
                        )
                    }
                    Some(Sym::Global(_)) => {
                        self.error(
                            codes::NOT_CALLABLE,
                            callee.span,
                            format!("`{name}` is a variable, not a fn"),
                        );
                        self.err_expr(span)
                    }
                    Some(Sym::Enum(_) | Sym::Alias(_)) => {
                        self.error(
                            codes::NOT_CALLABLE,
                            callee.span,
                            format!("`{name}` is a type and can't be called"),
                        );
                        self.err_expr(span)
                    }
                    None => self.prelude(name, args, hint, span),
                }
            }
            ast::ExprKind::Generic { base, args: types } => {
                let name = match &base.kind {
                    ast::ExprKind::Name(name) => name.as_str(),
                    _ => "",
                };
                if !matches!(name, "List" | "Set" | "Map") {
                    self.unsupported(span, "generic fns and types");
                    return self.err_expr(span);
                }
                if !args.is_empty() {
                    self.unsupported(span, "arguments when making a collection");
                    return self.err_expr(span);
                }
                let ty = ast::Type {
                    kind: ast::TypeKind::Named {
                        name: ast::Name {
                            text: name.to_string(),
                            span: base.span,
                        },
                        args: types.clone(),
                    },
                    span: callee.span,
                };
                let ty = self.resolve_ty(&ty);
                self.mk(ExprKind::Tuple(Vec::new()), ty, span)
            }
            ast::ExprKind::Field { base, name } => self.method(whole, base, name, args, hint),
            _ => {
                self.unsupported(span, "calling the result of an expression");
                self.err_expr(span)
            }
        }
    }

    /// Positional arguments only, and exactly `n` of them.
    fn positional_any<'b>(
        &mut self,
        what: &str,
        args: &'b [ast::Arg],
        n: usize,
        span: Span,
    ) -> Option<Vec<&'b ast::Expr>> {
        if let Some(named) = args.iter().find_map(|a| a.name.as_ref()) {
            let message = format!("`{what}` has no argument named `{}`", named.text);
            self.error(codes::UNKNOWN_ARG_NAME, named.span, message);
            return None;
        }
        if args.len() != n {
            let message = format!(
                "`{what}` takes {n} argument{}, but {} {} given",
                if n == 1 { "" } else { "s" },
                args.len(),
                if args.len() == 1 { "was" } else { "were" }
            );
            self.error(codes::WRONG_ARG_COUNT, span, message);
            return None;
        }
        Some(args.iter().map(|a| &a.value).collect())
    }

    fn one_arg(&mut self, what: &str, args: &[ast::Arg], want: &Ty, span: Span) -> Option<Expr> {
        let exprs = self.positional_any(what, args, 1, span)?;
        Some(self.expect(exprs[0], want))
    }

    /// Calls the user's fn `id`. Arguments are matched by position, then
    /// by name, then from defaults.
    fn user_call(
        &mut self,
        id: FnId,
        receiver: Option<Arg>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let params = self.sigs[id].params.clone();
        let name = self.sigs[id].decl.name.text.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; params.len()];
        let mut next = 0;
        let mut ok = true;
        for arg in args {
            let index = match &arg.name {
                Some(arg_name) => match params.iter().position(|p| p.name == arg_name.text) {
                    Some(index) => index,
                    None => {
                        let message =
                            format!("`{name}` has no parameter named `{}`", arg_name.text);
                        self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        ok = false;
                        continue;
                    }
                },
                None => {
                    while next < params.len() && given[next].is_some() {
                        next += 1;
                    }
                    next
                }
            };
            if index >= params.len() {
                let message = format!(
                    "`{name}` takes {} argument{}, but {} were given",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    args.len()
                );
                self.error(codes::WRONG_ARG_COUNT, span, message);
                ok = false;
                break;
            }
            given[index] = Some(arg);
        }
        let mut lowered: Vec<Arg> = receiver.into_iter().collect();
        for (param, arg) in params.iter().zip(given) {
            match arg {
                Some(arg) if param.mutable => {
                    if !arg.mutable {
                        let message =
                            format!("`{name}` changes `{}`, so pass it with `mut`", param.name);
                        self.error(codes::MISSING_MUT_AT_CALL, arg.value.span, message);
                        ok = false;
                    }
                    let place = self.place(&arg.value);
                    if place.ty == Ty::Bool && matches!(place.kind, PlaceKind::Index(..)) {
                        // The C++ back end packs `Bool` elements as bytes,
                        // which a `bool&` parameter can't refer to.
                        self.unsupported(arg.value.span, "a `Bool` element passed as `mut`");
                        ok = false;
                    }
                    if place.ty != param.ty && place.ty != Ty::Error && param.ty != Ty::Error {
                        self.mismatch(arg.value.span, &param.ty, &place.ty);
                        ok = false;
                    }
                    lowered.push(Arg::Mut(place));
                }
                Some(arg) => lowered.push(Arg::Value(self.expect(&arg.value, &param.ty))),
                None => match param.default {
                    Some(default) if !param.mutable => {
                        // A default is written next to the fn, so its
                        // spans are relative to the fn's item.
                        let saved = std::mem::replace(&mut self.offset, self.sigs[id].offset);
                        let value = self.expect(default, &param.ty);
                        self.offset = saved;
                        lowered.push(Arg::Value(value));
                    }
                    _ => {
                        if ok {
                            let message = format!("`{name}` needs the argument `{}`", param.name);
                            self.error(codes::WRONG_ARG_COUNT, span, message);
                        }
                        ok = false;
                    }
                },
            }
        }
        let ret = self.ret_of(id, span);
        if !ok {
            return self.err_expr(span);
        }
        self.mk(ExprKind::Call(id, lowered), ret, span)
    }

    /// `Edge(u: a, v: b, w: 5)` or `Edge(a, b, 5)`.
    fn construct(&mut self, id: StructId, args: &[ast::Arg], span: Span) -> Expr {
        let fields = self.structs[id].def.fields.clone();
        let decl = self.structs[id].decl;
        let name = decl.name.text.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; fields.len()];
        for (i, arg) in args.iter().enumerate() {
            let index = match &arg.name {
                Some(arg_name) => fields.iter().position(|(f, _)| *f == arg_name.text),
                None => (i < fields.len()).then_some(i),
            };
            match index {
                Some(index) => given[index] = Some(arg),
                None => {
                    let message = match &arg.name {
                        Some(arg_name) => format!("`{name}` has no field `{}`", arg_name.text),
                        None => format!(
                            "`{name}` has {} fields, but {} values were given",
                            fields.len(),
                            args.len()
                        ),
                    };
                    self.error(codes::WRONG_ARG_COUNT, arg.value.span, message);
                    return self.err_expr(span);
                }
            }
        }
        let mut values = Vec::new();
        for (i, ((field, ty), arg)) in fields.iter().zip(given).enumerate() {
            match (arg, &decl.fields[i].default) {
                (Some(arg), _) => values.push(self.expect(&arg.value, ty)),
                (None, Some(default)) => {
                    let saved = std::mem::replace(&mut self.offset, self.structs[id].offset);
                    values.push(self.expect(default, ty));
                    self.offset = saved;
                }
                (None, None) => {
                    let message = format!("`{name}` needs a value for its field `{field}`");
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    return self.err_expr(span);
                }
            }
        }
        self.mk(ExprKind::Tuple(values), Ty::Struct(id), span)
    }

    fn variant(&mut self, id: EnumId, tag: usize, args: &[ast::Arg], span: Span) -> Expr {
        let (name, fields) = self.enums[id].variants[tag].clone();
        let Some(exprs) = self.positional_any(&name, args, fields.len(), span) else {
            return self.err_expr(span);
        };
        let values = exprs
            .into_iter()
            .zip(&fields)
            .map(|(e, ty)| self.expect(e, ty))
            .collect();
        self.mk(ExprKind::Variant(tag, values), Ty::Enum(id), span)
    }

    /// Fns every file can call without declaring them.
    fn prelude(&mut self, name: &str, args: &[ast::Arg], hint: Option<&Ty>, span: Span) -> Expr {
        match name {
            "print" => {
                let mut digits = 6;
                let mut values = Vec::new();
                for arg in args {
                    match &arg.name {
                        Some(arg_name) if arg_name.text == "digits" => match arg.value.kind {
                            ast::ExprKind::Int(d) if (0..=18).contains(&d) => digits = d as u32,
                            _ => self.error(
                                codes::TYPE_MISMATCH,
                                arg.value.span,
                                "`digits` is a whole number from 0 to 18",
                            ),
                        },
                        Some(arg_name) => {
                            let message =
                                format!("`print` has no argument named `{}`", arg_name.text);
                            self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        }
                        None => values.push((self.expr(&arg.value, None), arg.value.span)),
                    }
                }
                let values = values
                    .into_iter()
                    .filter_map(|(value, at)| self.fmt_arg(value, digits, at))
                    .collect();
                self.mk(ExprKind::Print(values), Ty::Unit, span)
            }
            "read" => {
                if self.positional_any("read", args, 0, span).is_none() {
                    return self.err_expr(span);
                }
                let Some(want) = hint else {
                    self.error(codes::TYPE_MISMATCH, span, "can't tell what to read here");
                    self.help("`read()` reads what the type says: `let n: Int = read()`");
                    return self.err_expr(span);
                };
                match self.spec_of(want) {
                    Some(spec) => self.read(spec, span),
                    None => {
                        let message =
                            format!("`read()` can't tell how much a `{}` is", self.show(want));
                        self.error(codes::TYPE_MISMATCH, span, message);
                        self.help(
                            "declare the variable with its full type: `let a: [n]Int = read()`",
                        );
                        self.err_expr(span)
                    }
                }
            }
            "min" | "max" => self.min_max(name == "max", args, span),
            "sum" | "count" | "all" | "any" => {
                let kind = match name {
                    "sum" => Reduce::Sum,
                    "count" => Reduce::Count,
                    "all" => Reduce::All,
                    _ => Reduce::Any,
                };
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                self.reduce(kind, exprs[0], span)
            }
            "abs" => {
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                match value.ty {
                    Ty::Int | Ty::Float => {
                        let ty = value.ty.clone();
                        self.mk(ExprKind::Builtin(Builtin::Abs, vec![value]), ty, span)
                    }
                    Ty::Error => value,
                    _ => {
                        self.mismatch(exprs[0].span, &Ty::Int, &value.ty);
                        self.err_expr(span)
                    }
                }
            }
            "sqrt" => match self.one_arg(name, args, &Ty::Float, span) {
                Some(value) => self.mk(
                    ExprKind::Builtin(Builtin::Sqrt, vec![value]),
                    Ty::Float,
                    span,
                ),
                None => self.err_expr(span),
            },
            "floor" | "ceil" | "round" => {
                let op = match name {
                    "floor" => Builtin::Floor,
                    "ceil" => Builtin::Ceil,
                    _ => Builtin::Round,
                };
                match self.one_arg(name, args, &Ty::Float, span) {
                    Some(value) => self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span),
                    None => self.err_expr(span),
                }
            }
            "len" => {
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                self.len_of(value, span)
            }
            "sorted" => {
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                self.sorted(value, span)
            }
            "Int" | "Float" | "Char" | "Str" => {
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                self.convert(name, value, span)
            }
            _ => {
                let message = format!("`cpl run` doesn't know a fn named `{name}`");
                self.error(codes::UNRESOLVED_NAME, span, message);
                self.help("the standard library is not loaded yet; only the built-in fns exist");
                self.err_expr(span)
            }
        }
    }

    fn convert(&mut self, to: &str, value: Expr, span: Span) -> Expr {
        let from = value.ty.clone();
        let (cast, ty) = match (to, &from) {
            (_, Ty::Error) => return value,
            ("Int", Ty::Int) | ("Float", Ty::Float) | ("Char", Ty::Char) | ("Str", Ty::Str) => {
                return value
            }
            ("Int", Ty::Float) => (Cast::FloatToInt, Ty::Int),
            ("Int", Ty::Char) => (Cast::CharToInt, Ty::Int),
            ("Int", Ty::Pos(d)) => (Cast::Label(*d), Ty::Int),
            ("Int", Ty::Bound(_)) => (Cast::Ord, Ty::Int),
            ("Int", Ty::Bool) => {
                let one = self.int_lit(1, span);
                let zero = self.int_lit(0, span);
                return self.mk(
                    ExprKind::If(Box::new(value), Box::new(one), Box::new(zero)),
                    Ty::Int,
                    span,
                );
            }
            ("Float", Ty::Int) => (Cast::IntToFloat, Ty::Float),
            ("Char", Ty::Int) => (Cast::IntToChar, Ty::Char),
            ("Str", _) => {
                let Some(arg) = self.fmt_arg(value, 6, span) else {
                    return self.err_expr(span);
                };
                return self.mk(ExprKind::Format(vec![FmtPiece::Value(arg)]), Ty::Str, span);
            }
            _ => {
                let message = format!("a `{}` can't be turned into a `{to}`", self.show(&from));
                self.error(codes::TYPE_MISMATCH, span, message);
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Cast(cast, Box::new(value)), ty, span)
    }

    fn len_of(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Array { .. } | Ty::List(_) | Ty::Str | Ty::Map(..) | Ty::Set(_) => {
                self.mk(ExprKind::Builtin(Builtin::Len, vec![value]), Ty::Int, span)
            }
            Ty::Error => value,
            _ => {
                let message = format!("a `{}` has no length", self.show(&value.ty));
                self.error(codes::UNKNOWN_FIELD, span, message);
                self.err_expr(span)
            }
        }
    }

    fn sorted(&mut self, value: Expr, span: Span) -> Expr {
        match &value.ty {
            Ty::List(elem) => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                let ty = value.ty.clone();
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![value]), ty, span)
            }
            Ty::Set(elem) => {
                // A set is walked in order already.
                let ty = Ty::List(elem.clone());
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![value]), ty, span)
            }
            Ty::Error => value,
            other => {
                let message = format!("`sorted` takes a list, not a `{}`", self.show(other));
                self.error(codes::TYPE_MISMATCH, span, message);
                self.err_expr(span)
            }
        }
    }

    /// The spec of a type that says everything `read()` needs on its own.
    fn spec_of(&mut self, ty: &Ty) -> Option<Spec> {
        Some(match ty {
            Ty::Struct(id) => self.struct_spec(*id),
            Ty::Tuple(_) | Ty::Record(_) => {
                let fields = ty.fields().expect("a tuple or record");
                let specs: Option<Vec<Spec>> = fields.iter().map(|f| self.spec_of(f)).collect();
                Spec::Fields(ty.clone(), specs?)
            }
            Ty::Array { dims, elem } => {
                let dims: Option<Vec<DimSpec>> = dims
                    .iter()
                    .map(|d| match d {
                        Dim::Pos(d) => Some(DimSpec::Pos(*d)),
                        Dim::Bound(d) => Some(DimSpec::Bound(*d)),
                        Dim::Int => None,
                    })
                    .collect();
                Spec::Array {
                    dims: dims?,
                    elem: Box::new(self.spec_of(elem)?),
                }
            }
            other => Spec::Plain(other.clone()),
        })
    }

    /// `read()` of a value shaped like `spec`.
    pub(super) fn read(&mut self, spec: Spec, span: Span) -> Expr {
        fn unreadable(spec: &Spec) -> Option<Ty> {
            match spec {
                Spec::Plain(ty) => match ty {
                    Ty::Int | Ty::Float | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Error => None,
                    other => Some(other.clone()),
                },
                Spec::Int { .. } => None,
                Spec::Fields(_, fields) => fields.iter().find_map(unreadable),
                Spec::Array { elem, .. } => unreadable(elem),
                Spec::Opt(_) => Some(spec.ty()),
            }
        }
        if let Some(ty) = unreadable(&spec) {
            let message = format!("`read()` has no input format for a `{}`", self.show(&ty));
            self.error(codes::UNSUPPORTED, span, message);
            return self.err_expr(span);
        }
        let ty = spec.ty();
        self.mk(ExprKind::Read(Box::new(spec)), ty, span)
    }

    fn min_max(&mut self, is_max: bool, args: &[ast::Arg], span: Span) -> Expr {
        let name = if is_max { "max" } else { "min" };
        if let Some(named) = args.iter().find_map(|a| a.name.as_ref()) {
            let message = format!("`{name}` has no argument named `{}`", named.text);
            self.error(codes::UNKNOWN_ARG_NAME, named.span, message);
            return self.err_expr(span);
        }
        match args {
            [] => {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    span,
                    format!("`{name}` needs something to compare"),
                );
                self.err_expr(span)
            }
            [one] => self.reduce(
                if is_max { Reduce::Max } else { Reduce::Min },
                &one.value,
                span,
            ),
            _ => {
                let mut values: Vec<Expr> =
                    args.iter().map(|a| self.expr(&a.value, None)).collect();
                if values.iter().any(|v| v.ty == Ty::Error) {
                    return self.err_expr(span);
                }
                let ty = if values.iter().any(|v| v.ty == Ty::Float) {
                    Ty::Float
                } else {
                    values[0].ty.clone()
                };
                if matches!(ty, Ty::Opt(_)) {
                    self.unsupported(span, &format!("`{name}` of optionals"));
                    return self.err_expr(span);
                }
                if !self.ordered(&ty) {
                    let message = format!("a `{}` has no order for `{name}`", self.show(&ty));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                for (i, arg) in args.iter().enumerate() {
                    let value = std::mem::replace(&mut values[i], self.err_expr(span));
                    values[i] = self.coerce(value, &ty, arg.value.span);
                }
                let op = if is_max { Builtin::Max } else { Builtin::Min };
                self.mk(ExprKind::Builtin(op, values), ty, span)
            }
        }
    }

    /// `sum(a[v] for v in V)`, `max(xs)`, `any(...)`: a loop that folds
    /// the values into one.
    fn reduce(&mut self, kind: Reduce, arg: &ast::Expr, span: Span) -> Expr {
        let at = self.span(span);
        let (frames, value) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = match kind {
                    Reduce::Count | Reduce::All | Reduce::Any => self.expect(body, &Ty::Bool),
                    _ => self.expr(body, None),
                };
                (frames, value)
            }
            _ => {
                let seq = self.expr(arg, None);
                let elem = match &seq.ty {
                    Ty::Array { elem, .. } | Ty::List(elem) | Ty::Set(elem) => (**elem).clone(),
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(other)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let local = |slot: Slot, ty: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let place = |slot: Slot, ty: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let lit = |kind: ExprKind, ty: Ty| Expr { kind, ty, span: at };
        let mut stmts = Vec::new();
        let (acc, ty, inner, after): (Slot, Ty, Vec<Stmt>, Vec<Stmt>) = match kind {
            Reduce::Sum => {
                let ty = value.ty.clone();
                let zero = match ty {
                    Ty::Int => ExprKind::Int(0),
                    Ty::Float => ExprKind::Float(0.0),
                    Ty::Error => {
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                    _ => {
                        let message = format!("`sum` adds numbers, not `{}`", self.show(&ty));
                        self.error(codes::BAD_OPERANDS, arg.span, message);
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                };
                let acc = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, Some(lit(zero, ty.clone()))));
                let add = Stmt::Assign(place(acc, &ty), AssignOp::Binary(BinOp::Add), value);
                (acc, ty, vec![add], Vec::new())
            }
            Reduce::Count => {
                let acc = self.temp(Ty::Int);
                stmts.push(Stmt::Let(acc, Some(lit(ExprKind::Int(0), Ty::Int))));
                let add = Stmt::Assign(
                    place(acc, &Ty::Int),
                    AssignOp::Binary(BinOp::Add),
                    lit(ExprKind::Int(1), Ty::Int),
                );
                let test = Stmt::If(value, Block { stmts: vec![add] }, Block::default());
                (acc, Ty::Int, vec![test], Vec::new())
            }
            Reduce::All | Reduce::Any => {
                let all = kind == Reduce::All;
                let acc = self.temp(Ty::Bool);
                stmts.push(Stmt::Let(acc, Some(lit(ExprKind::Bool(all), Ty::Bool))));
                let set = Stmt::Assign(
                    place(acc, &Ty::Bool),
                    AssignOp::Set,
                    lit(ExprKind::Bool(!all), Ty::Bool),
                );
                // Leave every loop at the first value that decides it.
                let test = if all {
                    lit(ExprKind::Unary(UnOp::Not, Box::new(value)), Ty::Bool)
                } else {
                    value
                };
                (
                    acc,
                    Ty::Bool,
                    vec![Stmt::If(
                        test,
                        Block {
                            stmts: vec![set, Stmt::Break(None)],
                        },
                        Block::default(),
                    )],
                    Vec::new(),
                )
            }
            Reduce::Min | Reduce::Max => {
                let ty = value.ty.clone();
                if ty == Ty::Error {
                    self.close_loops(frames, Vec::new());
                    return self.err_expr(span);
                }
                if matches!(ty, Ty::Opt(_)) || !self.ordered(&ty) {
                    let message = format!("a `{}` has no order to compare by", self.show(&ty));
                    self.error(codes::BAD_OPERANDS, arg.span, message);
                    self.close_loops(frames, Vec::new());
                    return self.err_expr(span);
                }
                let acc = self.temp(ty.clone());
                let have = self.temp(Ty::Bool);
                let cur = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, None));
                stmts.push(Stmt::Let(have, Some(lit(ExprKind::Bool(false), Ty::Bool))));
                let op = if kind == Reduce::Max {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                let inner = vec![
                    Stmt::Let(cur, Some(value)),
                    Stmt::If(
                        local(have, &Ty::Bool),
                        Block {
                            stmts: vec![Stmt::Assign(place(acc, &ty), op, local(cur, &ty))],
                        },
                        Block {
                            stmts: vec![
                                Stmt::Assign(place(acc, &ty), AssignOp::Set, local(cur, &ty)),
                                Stmt::Assign(
                                    place(have, &Ty::Bool),
                                    AssignOp::Set,
                                    lit(ExprKind::Bool(true), Ty::Bool),
                                ),
                            ],
                        },
                    ),
                ];
                let word = if kind == Reduce::Max { "max" } else { "min" };
                let empty = Stmt::If(
                    lit(
                        ExprKind::Unary(UnOp::Not, Box::new(local(have, &Ty::Bool))),
                        Ty::Bool,
                    ),
                    Block {
                        stmts: vec![Stmt::Trap(format!("`{word}` of an empty sequence"), at)],
                    },
                    Block::default(),
                );
                (acc, ty, inner, vec![empty])
            }
        };
        let (mut loops, outer) = self.close_loops(frames, inner);
        if matches!(kind, Reduce::All | Reduce::Any) {
            retarget_break(&mut loops, outer, 0);
        }
        stmts.extend(loops);
        stmts.extend(after);
        stmts.push(Stmt::Give(local(acc, &ty)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    // ----- members -----

    /// `base.name` that is not being called.
    pub(super) fn field(&mut self, whole: &ast::Expr, base: &ast::Expr, name: &ast::Name) -> Expr {
        let span = whole.span;
        match self.sym(base) {
            Some(Sym::Domain(id)) => {
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                return match name.text.as_str() {
                    "len" | "size" => len,
                    "first" => {
                        let zero = self.int_lit(0, span);
                        self.mk(
                            ExprKind::Cast(Cast::FromOrd(id), Box::new(zero)),
                            Ty::Pos(id),
                            span,
                        )
                    }
                    "last" => {
                        let last = self.int_add(len, -1);
                        self.mk(
                            ExprKind::Cast(Cast::FromOrd(id), Box::new(last)),
                            Ty::Pos(id),
                            span,
                        )
                    }
                    "start" => {
                        let zero = self.int_lit(0, span);
                        self.mk(
                            ExprKind::Cast(Cast::Trust, Box::new(zero)),
                            Ty::Bound(id),
                            span,
                        )
                    }
                    "end" => self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(len)),
                        Ty::Bound(id),
                        span,
                    ),
                    other => {
                        self.unsupported(name.span, &format!("`.{other}` of a domain as a value"));
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) if self.enums[id].variants[tag].1.is_empty() => {
                        self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span)
                    }
                    Some(_) => {
                        let message = format!(
                            "the variant `{}` holds values: write `{}(...)`",
                            name.text, name.text
                        );
                        self.error(codes::WRONG_ARG_COUNT, span, message);
                        self.err_expr(span)
                    }
                    None => {
                        let message =
                            format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            _ => {}
        }
        if let ast::ExprKind::Name(type_name) = &base.kind {
            if type_name == "Int" && self.lookup(type_name).is_none() {
                match name.text.as_str() {
                    "MAX" => return self.mk(ExprKind::Int(i64::MAX), Ty::Int, span),
                    "MIN" => return self.mk(ExprKind::Int(INT_MIN), Ty::Int, span),
                    _ => {}
                }
            }
        }
        let value = self.expr(base, None);
        if let Some((index, ty)) = self.field_of(&value.ty, &name.text) {
            return self.mk(ExprKind::Field(Box::new(value), index), ty, span);
        }
        let ty = value.ty.clone();
        let builtin = |lw: &mut Self, op: Builtin, args: Vec<Expr>, ty: Ty| {
            lw.mk(ExprKind::Builtin(op, args), ty, span)
        };
        match (&ty, name.text.as_str()) {
            (Ty::Error, _) => value,
            // A method without parameters reads like a field.
            (Ty::Struct(id), method) if self.structs[*id].methods.contains_key(method) => {
                let fn_id = self.structs[*id].methods[method];
                self.method_call(fn_id, Some((base, value)), &[], span)
            }
            (Ty::Pos(_) | Ty::Bound(_), "ord") => self.cast(Cast::Ord, value, Ty::Int),
            (Ty::Pos(_), "rank") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                self.int_add(ord, 1)
            }
            (Ty::Pos(d), "next") => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![value, len],
                    Ty::opt(ty.clone()),
                )
            }
            (Ty::Pos(_), "prev") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(ty.clone()))
            }
            (Ty::Pos(d), "before") => self.cast(Cast::Trust, value, Ty::Bound(*d)),
            (Ty::Pos(d), "after") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let next = self.int_add(ord, 1);
                self.cast(Cast::Trust, next, Ty::Bound(*d))
            }
            (Ty::Bound(d), "left") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(Ty::Pos(*d)))
            }
            (Ty::Bound(d), "right") => {
                // The position after gap `b` is `b` itself, if there is one.
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let before = self.int_add(ord, -1);
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![before, len],
                    Ty::opt(Ty::Pos(*d)),
                )
            }
            (Ty::Array { .. } | Ty::List(_) | Ty::Str | Ty::Map(..) | Ty::Set(_), "len") => {
                self.len_of(value, span)
            }
            (Ty::Array { .. } | Ty::List(_) | Ty::Str | Ty::Map(..) | Ty::Set(_), "is_empty") => {
                let len = self.len_of(value, span);
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::List(elem), "first") => {
                builtin(self, Builtin::First, vec![value], Ty::Opt(elem.clone()))
            }
            (Ty::List(elem), "last") => {
                builtin(self, Builtin::Last, vec![value], Ty::Opt(elem.clone()))
            }
            (Ty::Opt(_), "is_some") => self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span),
            (Ty::Opt(_), "is_none") => {
                let some = self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span);
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span)
            }
            _ => {
                let message = format!(
                    "a `{}` has no member `{}` that `cpl run` knows",
                    self.show(&ty),
                    name.text
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                self.err_expr(span)
            }
        }
    }

    /// Calls method `fn_id`. With a receiver, `self` is the caller's own
    /// variable when the method changes it.
    fn method_call(
        &mut self,
        fn_id: FnId,
        receiver: Option<(&ast::Expr, Expr)>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let name = self.sigs[fn_id].decl.name.text.clone();
        let receiver = match (self.sigs[fn_id].receiver, receiver) {
            (Some(true), Some((base, _))) => Some(Arg::Mut(self.place(base))),
            (Some(false), Some((_, value))) => Some(Arg::Value(value)),
            (None, None) => None,
            (Some(_), None) => {
                let message = format!("`{name}` is called on a value: `x.{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
            (None, Some(_)) => {
                let message = format!("`{name}` takes no `self`: call it on the type");
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
        };
        self.user_call(fn_id, receiver, args, span)
    }

    /// `base.name(args)`.
    fn method(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        let _ = hint;
        match self.sym(base) {
            Some(Sym::Struct(id)) => {
                return match self.structs[id].methods.get(&name.text).copied() {
                    Some(fn_id) => self.method_call(fn_id, None, args, span),
                    None => {
                        let message =
                            format!("`{}` has no fn `{}`", self.structs[id].def.name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) => self.variant(id, tag, args, span),
                    None => {
                        let message =
                            format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Domain(id)) if name.text == "at" => {
                return match self.one_arg("at", args, &Ty::Int, span) {
                    Some(value) => self.mk(
                        ExprKind::Cast(Cast::FromOrd(id), Box::new(value)),
                        Ty::Pos(id),
                        span,
                    ),
                    None => self.err_expr(span),
                };
            }
            _ => {}
        }
        let value = self.expr(base, None);
        let ty = value.ty.clone();
        let method = name.text.as_str();
        if let Ty::Struct(id) = &ty {
            if let Some(fn_id) = self.structs[*id].methods.get(method).copied() {
                return self.method_call(fn_id, Some((base, value)), args, span);
            }
        }
        // Members that read like fields can be called too: `xs.len()`.
        if args.is_empty()
            && matches!(method, "len" | "is_empty" | "first" | "last")
            && !matches!(ty, Ty::Set(_) | Ty::Struct(_))
        {
            return self.field(whole, base, name);
        }
        let elem = match &ty {
            Ty::List(elem) | Ty::Set(elem) => (**elem).clone(),
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Map(key, _) => (**key).clone(),
            Ty::Str => Ty::Char,
            _ => Ty::Error,
        };
        let mutate = |lw: &mut Self, op: Mutation, args: Vec<Expr>, result: Ty| {
            let place = lw.place(base);
            lw.mk(ExprKind::Mutate(op, Box::new(place), args), result, span)
        };
        match (&ty, method) {
            (Ty::Error, _) => {
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.err_expr(span)
            }
            (Ty::List(_) | Ty::Str, "push") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => mutate(self, Mutation::Push, vec![arg], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "pop") => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Pop, Vec::new(), Ty::opt(elem)),
                None => self.err_expr(span),
            },
            (Ty::Set(_), "insert") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => mutate(self, Mutation::Insert, vec![arg], Ty::Bool),
                None => self.err_expr(span),
            },
            (Ty::Set(_) | Ty::Map(..), "remove") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => mutate(self, Mutation::Remove, vec![arg], Ty::Bool),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Set(_) | Ty::Map(..), "clear") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, Mutation::Clear, Vec::new(), Ty::Unit),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_), "reverse") => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Reverse, Vec::new(), Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "sort") => self.sort(base, &elem, args, span),
            (Ty::List(_) | Ty::Set(_), "sorted") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.sorted(value, span),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_), "reversed") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    ty.clone(),
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Set(_) | Ty::Map(..) | Ty::Array { .. } | Ty::Str, "contains") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.mk(
                        ExprKind::Builtin(Builtin::Contains, vec![value, arg]),
                        Ty::Bool,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Map(_, val), "get") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => self.mk(
                    ExprKind::Builtin(Builtin::MapGet, vec![value, arg]),
                    Ty::Opt(val.clone()),
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::Set(_), "min" | "max") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    let op = if method == "min" {
                        Builtin::SetMin
                    } else {
                        Builtin::SetMax
                    };
                    self.mk(ExprKind::Builtin(op, vec![value]), Ty::opt(elem), span)
                }
                None => self.err_expr(span),
            },
            _ => {
                let message = format!(
                    "a `{}` has no method `{method}` that `cpl run` knows",
                    self.show(&ty)
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.err_expr(span)
            }
        }
    }

    /// `xs.sort()` or `xs.sort(by: x => key)`.
    fn sort(&mut self, base: &ast::Expr, elem: &Ty, args: &[ast::Arg], span: Span) -> Expr {
        let place = self.place(base);
        let key = match args {
            [] => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    self.help("sort by a key: `xs.sort(by: x => x.w)`");
                    return self.err_expr(span);
                }
                None
            }
            [ast::Arg {
                name: Some(by),
                value:
                    ast::Expr {
                        kind: ast::ExprKind::Lambda { params, body },
                        ..
                    },
                ..
            }] if by.text == "by" && params.len() == 1 => {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let slot = self.loop_var(&params[0], elem.clone(), &mut pre);
                let key = self.expr(body, None);
                self.cx.scopes.pop();
                if !pre.is_empty() {
                    self.unsupported(params[0].span, "a pattern as the parameter of a sort key");
                    return self.err_expr(span);
                }
                if key.ty != Ty::Error && (matches!(key.ty, Ty::Opt(_)) || !self.ordered(&key.ty)) {
                    let message = format!("a `{}` has no order to sort by", self.show(&key.ty));
                    self.error(codes::BAD_OPERANDS, body.span, message);
                    return self.err_expr(span);
                }
                Some((slot, Box::new(key)))
            }
            _ => {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    span,
                    "`sort` takes nothing, or a key: `xs.sort(by: x => x.w)`",
                );
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Sort(Box::new(place), key), Ty::Unit, span)
    }
}

/// Turns the `break` that ends an `all`/`any` early into one that leaves
/// the outermost loop of the comprehension.
fn retarget_break(stmts: &mut [Stmt], outer: LoopId, depth: usize) {
    for stmt in stmts {
        match stmt {
            Stmt::For(_, _, _, body) => retarget_break(&mut body.stmts, outer, depth + 1),
            Stmt::If(_, then, _) => retarget_break(&mut then.stmts, outer, depth),
            Stmt::Break(target @ None) if depth > 1 => *target = Some(outer),
            _ => {}
        }
    }
}
