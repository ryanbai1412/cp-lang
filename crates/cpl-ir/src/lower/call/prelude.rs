//! The prelude's free fns: conversions, printing, sorting and reading.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn variant(
        &mut self,
        id: EnumId,
        tag: usize,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let (name, fields) = self.enums[id].variants[tag].clone();
        let Some(exprs) = self.positional_any(&name, args, fields.len(), span) else {
            return self.err_expr(span);
        };
        let values: Vec<Expr> = exprs
            .into_iter()
            .zip(&fields)
            .map(|(e, ty)| self.expect(e, ty))
            .collect();
        if let Some(&d) = self.enum_to_sum_domain.get(&id) {
            return self.encode_sum_variant(d, tag, values, span);
        }
        self.mk(ExprKind::Variant(tag, values), Ty::Enum(id), span)
    }

    /// Fns every file can call without declaring them.
    pub(in crate::lower) fn prelude(
        &mut self,
        name: &str,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        match name {
            "print" => {
                let mut digits = self.io_precision.unwrap_or_else(|| {
                    match self.context("io", "precision").map(|v| v.kind) {
                        Some(ast::ExprKind::Int(d)) if (0..=18).contains(&d) => d as u32,
                        _ => crate::FLOAT_DIGITS,
                    }
                });
                let mut values = Vec::new();
                let mut sep = if let Some(s) = &self.io_sep {
                    Some(self.mk(ExprKind::Str(s.as_bytes().to_vec()), Ty::Str, span))
                } else if self.cx.uses_io {
                    let gid = self.ensure_io_global();
                    let io_ty = self.globals[gid].ty.clone();
                    let io_expr = self.mk(ExprKind::Global(gid), io_ty, span);
                    Some(self.mk(ExprKind::Field(Box::new(io_expr), 0), Ty::Str, span))
                } else {
                    None
                };
                let mut none_text = None;
                let mut end = None;
                for arg in args {
                    match &arg.name {
                        // `end: " "` is written in place of the newline.
                        Some(arg_name) if arg_name.text == "end" => {
                            end = Some(Box::new(self.expect(&arg.value, &Ty::Str)));
                        }
                        // `sep: "\n"` goes between the arguments and between
                        // the items of a list; `none: "-1"` is what a `none`
                        // is written as.
                        Some(arg_name) if arg_name.text == "sep" => {
                            sep = Some(self.expect(&arg.value, &Ty::Str));
                        }
                        Some(arg_name) if arg_name.text == "none" => {
                            let value = self.expr(&arg.value, None);
                            none_text = if value.ty == Ty::Str {
                                Some(value)
                            } else {
                                let at = arg.value.span;
                                self.fmt_arg(value, digits, at).map(|piece| {
                                    self.mk(
                                        ExprKind::Format(vec![FmtPiece::Value(piece)]),
                                        Ty::Str,
                                        at,
                                    )
                                })
                            };
                        }
                        Some(arg_name)
                            if arg_name.text == "digits" || arg_name.text == "precision" =>
                        {
                            match arg.value.kind {
                                ast::ExprKind::Int(d) if (0..=18).contains(&d) => digits = d as u32,
                                _ => self.error(
                                    codes::TYPE_MISMATCH,
                                    arg.value.span,
                                    "`precision` is a whole number from 0 to 18",
                                ),
                            }
                        }
                        Some(arg_name) => {
                            let message =
                                format!("`print` has no argument named `{}`", arg_name.text);
                            self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        }
                        None => values.push((self.expr(&arg.value, None), arg.value.span)),
                    }
                }
                if sep.is_none() {
                    if let Some(value) = self.context("io", "sep") {
                        sep = Some(self.expect(&value, &Ty::Str));
                    }
                }
                // Every argument is worked out before anything is written:
                // one that prints (`print("A", side())`) has its output
                // ahead of this line, in both back ends. Arguments after
                // the last call are read where they are.
                let calls = |e: &Expr| {
                    let mut found = false;
                    crate::walk::deep_expr(e, &mut |inner| {
                        found |= matches!(inner.kind, ExprKind::Call(..) | ExprKind::CallExpr(..));
                    });
                    found
                };
                let mut ahead = Vec::new();
                if let Some(last) = values.iter().rposition(|(value, _)| calls(value)) {
                    for (value, _) in &mut values[..=last] {
                        let literal = matches!(
                            value.kind,
                            ExprKind::Int(_)
                                | ExprKind::Float(_)
                                | ExprKind::Bool(_)
                                | ExprKind::Char(_)
                                | ExprKind::Str(_)
                        );
                        if literal || value.ty == Ty::Unit || value.ty == Ty::Error {
                            continue;
                        }
                        let slot = self.temp(value.ty.clone());
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: value.ty.clone(),
                            span: value.span,
                        };
                        ahead.push(Stmt::Let(slot, Some(std::mem::replace(value, held))));
                    }
                }
                values = values
                    .into_iter()
                    .map(|(value, at)| (self.in_base(value, at), at))
                    .collect();
                // A range is a `Seq`: it prints its elements (R10.3.1).
                let ranged = values.iter().any(|(v, _)| matches!(v.ty, Ty::Range(_)));
                if sep.is_some() || none_text.is_some() || ranged {
                    values = values
                        .into_iter()
                        .map(|(value, at)| {
                            (
                                self.print_piece(value, sep.as_ref(), none_text.as_ref(), at),
                                at,
                            )
                        })
                        .collect();
                }
                let mut values: Vec<FmtArg> = values
                    .into_iter()
                    .filter_map(|(value, at)| self.fmt_arg(value, digits, at))
                    .collect();
                if let Some(sep) = sep {
                    // One text, so the separator is the only thing between.
                    let mut pieces = Vec::new();
                    for (i, arg) in values.drain(..).enumerate() {
                        if i > 0 {
                            pieces.push(FmtPiece::Value(FmtArg {
                                value: sep.clone(),
                                digits,
                            }));
                        }
                        pieces.push(FmtPiece::Value(arg));
                    }
                    let line = self.mk(ExprKind::Format(pieces), Ty::Str, span);
                    values.push(FmtArg {
                        value: line,
                        digits,
                    });
                }
                let print = self.mk(ExprKind::Print(values, end), Ty::Unit, span);
                if ahead.is_empty() {
                    return print;
                }
                ahead.push(Stmt::Expr(print));
                ahead.push(Stmt::Give(self.mk(ExprKind::Unit, Ty::Unit, span)));
                self.mk(ExprKind::Do(Block { stmts: ahead }), Ty::Unit, span)
            }
            // `debug(i, best)`: shown on a local run, and a checkpoint
            // of its trace. A string literal names the checkpoint.
            "debug" => {
                let mut values = Vec::new();
                for arg in args {
                    if let Some(arg_name) = &arg.name {
                        let message = format!("`debug` has no argument named `{}`", arg_name.text);
                        self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        continue;
                    }
                    let text = self.source(arg.value.span);
                    let label = matches!(arg.value.kind, ast::ExprKind::Str(_));
                    let value = self.expr(&arg.value, None);
                    if value.ty == Ty::Unit {
                        self.error(
                            codes::TYPE_MISMATCH,
                            arg.value.span,
                            "this has no value to show",
                        );
                        continue;
                    }
                    values.push(DebugArg { text, value, label });
                }
                let line = self.line(span);
                self.mk(ExprKind::Debug(line, values), Ty::Unit, span)
            }
            // `let k: Int = read_output()` in a `@check`: the next
            // thing the solution printed.
            "read_output" => {
                if self.aside != Some(Aside::Checker) {
                    self.error(
                        codes::UNRESOLVED_NAME,
                        span,
                        "`read_output()` reads what the solution printed, which only a `@check` fn sees",
                    );
                    return self.err_expr(span);
                }
                if !args.is_empty() {
                    self.error(
                        codes::WRONG_ARG_COUNT,
                        span,
                        "`read_output` takes no arguments",
                    );
                    return self.err_expr(span);
                }
                let Some(want) = hint.filter(|want| **want != Ty::Unit) else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "`read_output` can't tell what to read",
                    );
                    self.help("say what the solution printed: `let k: Int = read_output()`");
                    return self.err_expr(span);
                };
                let Some(spec) = self.spec_of(want) else {
                    let message = format!(
                        "`read_output` can't tell how much a `{}` is",
                        self.show(want)
                    );
                    self.error(codes::TYPE_MISMATCH, span, message);
                    return self.err_expr(span);
                };
                self.read_printed(spec, span)
            }
            // `let r: Int = ask("?", x)`: the query goes out, the answer
            // comes back as the type says. `ask("!", x)` only prints.
            "ask" => {
                self.interactive = true;
                let mut values = Vec::new();
                let mut limit = None;
                for arg in args {
                    match &arg.name {
                        // `limit: K`: the judge allows `K` queries, and
                        // one more fails here as it would there.
                        Some(name) if name.text == "limit" => {
                            limit = Some(self.expect(&arg.value, &Ty::Int));
                        }
                        Some(name) => {
                            let message = format!("`ask` has no argument named `{}`", name.text);
                            self.error(codes::UNKNOWN_ARG_NAME, name.span, message);
                        }
                        None => values.push((self.expr(&arg.value, None), arg.value.span)),
                    }
                }
                let values: Vec<FmtArg> = values
                    .into_iter()
                    .filter_map(|(value, at)| self.fmt_arg(value, crate::FLOAT_DIGITS, at))
                    .collect();
                let query = self.mk(ExprKind::Print(values, None), Ty::Unit, span);
                let counted = limit.map(|limit| {
                    self.mk(
                        ExprKind::Builtin(Builtin::CountQuery, vec![limit]),
                        Ty::Unit,
                        span,
                    )
                });
                let Some(want) = hint.filter(|want| **want != Ty::Unit) else {
                    let Some(counted) = counted else {
                        return query;
                    };
                    return self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                        out.push(Stmt::Expr(counted));
                        out.push(Stmt::Expr(query));
                        out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                    });
                };
                let Some(spec) = self.spec_of(want) else {
                    let message = format!("`ask` can't tell how much a `{}` is", self.show(want));
                    self.error(codes::TYPE_MISMATCH, span, message);
                    self.help("give the answer its full type: `let r: Int = ask(\"?\", x)`");
                    return self.err_expr(span);
                };
                let want = want.clone();
                self.do_expr(Some(&want), span, |lw, out| {
                    out.extend(counted.map(Stmt::Expr));
                    out.push(Stmt::Expr(query));
                    let answer = lw.read(spec, span);
                    out.push(Stmt::Give(answer));
                })
            }
            "read" => {
                // `read(1..=n)` reads an `Int` and checks it is in the range.
                if let [ast::Arg {
                    name: None,
                    mutable: false,
                    value,
                }] = args
                {
                    if let ast::ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    } = &value.kind
                    {
                        let lo = self.expect(lo, &Ty::Int);
                        let mut hi = self.expect(hi, &Ty::Int);
                        if !inclusive {
                            hi = self.int_add(hi, -1);
                        }
                        let spec = Spec::Int {
                            lo: Some(Box::new(lo)),
                            hi: Some(Box::new(hi)),
                            text: self.source(value.span),
                        };
                        return self.read(spec, span);
                    }
                }
                if let ([arg], Some(Ty::List(elem))) = (args, hint.as_ref()) {
                    if arg.name.is_none() {
                        self.read_count_arg(&arg.value, elem, span);
                        return self.err_expr(span);
                    }
                }
                if !args.iter().all(|a| {
                    a.name
                        .as_ref()
                        .is_some_and(|n| matches!(n.text.as_str(), "gen" | "none" | "base"))
                }) && self.positional_any("read", args, 0, span).is_none()
                {
                    return self.err_expr(span);
                }
                let Some(want) = hint else {
                    self.error(codes::TYPE_MISMATCH, span, "can't tell what to read here");
                    self.help("`read()` reads what the type says: `let n: Int = read()`");
                    return self.err_expr(span);
                };
                match self.spec_of(want) {
                    Some(spec) => self.read_with_args(spec, args, span),
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
            "read_line" => {
                if self.positional_any("read_line", args, 0, span).is_none() {
                    return self.err_expr(span);
                }
                self.mk(ExprKind::Read(Box::new(Spec::Line)), Ty::Str, span)
            }
            "min" | "max" => self.min_max(name == "max", args, span),
            "first" => {
                if let Some(walk) = self.tested_walk(name, args, true) {
                    return self.lower_first(&walk, span);
                }
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                self.lower_first(exprs[0], span)
            }
            "sum" | "count" | "all" | "any" => {
                let kind = match name {
                    "sum" => Reduce::Sum,
                    "count" => Reduce::Count,
                    "all" => Reduce::All,
                    _ => Reduce::Any,
                };
                if let Some(walk) = self.tested_walk(name, args, false) {
                    return self.reduce(kind, &walk, span);
                }
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
                    Ty::Int | Ty::I128 | Ty::Float => {
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
            "sqrt" | "atan2" | "sin" | "cos" | "tan" | "asin" | "acos" | "exp" | "ln" | "hypot" => {
                self.float_fn(name, args, span)
            }
            // A program that flushes is read by someone who answers: it
            // reads as it goes.
            "flush" => match self.positional_any(name, args, 0, span) {
                Some(_) => {
                    self.interactive = true;
                    self.mk(
                        ExprKind::Builtin(Builtin::Flush, Vec::new()),
                        Ty::Unit,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            "popcount" | "ctz" | "clz" | "bit_width" | "bit_ceil" => {
                let op = match name {
                    "popcount" => Builtin::Popcount,
                    "ctz" => Builtin::Ctz,
                    "clz" => Builtin::Clz,
                    "bit_width" => Builtin::BitWidth,
                    _ => Builtin::BitCeil,
                };
                // The bits of a `U64` are counted as they are.
                let value = match self.positional_any(name, args, 1, span) {
                    Some(exprs) if op != Builtin::BitCeil => {
                        let value = self.expr(exprs[0], None);
                        match value.ty {
                            Ty::U64 => value,
                            _ => self.coerce(value, &Ty::Int, exprs[0].span),
                        }
                    }
                    Some(exprs) => self.expect(exprs[0], &Ty::Int),
                    None => return self.err_expr(span),
                };
                self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span)
            }
            "swap" => {
                if args.len() != 2 {
                    self.error(codes::WRONG_ARG_COUNT, span, "`swap` takes 2 arguments");
                    return self.err_expr(span);
                }
                let pa = self.place(&args[0].value);
                let pb = self.place(&args[1].value);
                let ty = pa.ty.clone();
                if pb.ty != ty && ty != Ty::Error && pb.ty != Ty::Error {
                    self.mismatch(args[1].value.span, &ty, &pb.ty);
                    return self.err_expr(span);
                }
                // Three moves, so two lists trade places in O(1).
                self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                    let pa = lw.settle_indexes(pa, out);
                    let pb = lw.settle_indexes(pb, out);
                    let tmp = lw.temp(ty.clone());
                    let held = Place {
                        kind: PlaceKind::Local(tmp),
                        ty: ty.clone(),
                        span: lw.span(span),
                    };
                    let take = |lw: &mut Self, place: &Place| {
                        lw.mk(
                            ExprKind::Mutate(Mutation::Take, Box::new(place.clone()), Vec::new()),
                            ty.clone(),
                            span,
                        )
                    };
                    let from_a = take(lw, &pa);
                    out.push(Stmt::Let(tmp, Some(from_a)));
                    let from_b = take(lw, &pb);
                    out.push(Stmt::Assign(pa, AssignOp::Set, from_b));
                    let from_tmp = take(lw, &held);
                    out.push(Stmt::Assign(pb, AssignOp::Set, from_tmp));
                    out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                })
            }
            "gen" => self.gen_call(None, args, span),
            "rand" => match self.positional_any(name, args, 2, span) {
                Some(exprs) => {
                    let lo = self.expect(exprs[0], &Ty::Int);
                    let hi = self.expect(exprs[1], &Ty::Int);
                    self.mk(
                        ExprKind::Builtin(Builtin::Rand, vec![lo, hi]),
                        Ty::Int,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            "shuffle" => {
                if args.len() != 1 {
                    self.error(
                        codes::WRONG_ARG_COUNT,
                        span,
                        "`shuffle` takes 1 list argument",
                    );
                    return self.err_expr(span);
                }
                let place = self.place(&args[0].value);
                self.mk(
                    ExprKind::Mutate(Mutation::Shuffle, Box::new(place), Vec::new()),
                    Ty::Unit,
                    span,
                )
            }
            "copy" => match self.positional_any(name, args, 1, span) {
                Some(exprs) => {
                    let val = self.expr(exprs[0], hint);
                    let ty = val.ty.clone();
                    self.mk(ExprKind::Builtin(Builtin::Copy, vec![val]), ty, span)
                }
                None => self.err_expr(span),
            },
            "some" | "Some" => match self.positional_any(name, args, 1, span) {
                Some(exprs) => self.some_of(exprs[0], hint, span),
                None => self.err_expr(span),
            },
            "laws_hold" => self.lower_laws_hold(args, span),
            "sort" => {
                if args.is_empty() {
                    self.error(codes::WRONG_ARG_COUNT, span, "`sort` takes a list or array");
                    return self.err_expr(span);
                }
                let target = &args[0].value;
                let place = self.place(target);
                let elem = match &place.ty {
                    Ty::List(e) | Ty::Array { elem: e, .. } => (**e).clone(),
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!(
                            "`sort` orders a list or an array, not a `{}`",
                            self.show(other)
                        );
                        self.error(codes::BAD_OPERANDS, target.span, message);
                        if matches!(other, Ty::Range(_)) {
                            self.help("a range has no cells to move; `sorted(r, by: ..)` gives the list in order, and `[x for x in r]` the list");
                        }
                        return self.err_expr(span);
                    }
                };
                self.sort_place(place, &elem, &args[1..], span)
            }
            // Toward zero: what `Int(x)` used to mean without saying so.
            "trunc" => match self.one_arg(name, args, &Ty::Float, span) {
                Some(value) => self.cast(Cast::FloatToInt, value, Ty::Int),
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
            "adjacency" => self.lower_adjacency(args, span),
            "sorted" => {
                if args.is_empty() {
                    self.error(codes::WRONG_ARG_COUNT, span, "`sorted` takes a sequence");
                    return self.err_expr(span);
                }
                let value = match self.walked(&args[0].value) {
                    Some(values) => self.expr(&values, None),
                    None => self.expr(&args[0].value, None),
                };
                if args.len() == 1 {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, &args[1..], span)
                }
            }
            "argmax" | "argmin" if args.len() == 1 && self.lookup(name).is_none() => {
                self.arg_extreme(name == "argmax", &args[0].value, span)
            }
            "runs" => self.lower_runs(args, span),
            "counts" => self.lower_counts(args, span),
            "prefix" => self.lower_prefix(args, span),
            "List" => {
                if args.is_empty() {
                    let ty = match hint {
                        Some(ty @ Ty::List(_)) => ty.clone(),
                        _ => Ty::list(Ty::Int),
                    };
                    return self.mk(ExprKind::Tuple(Vec::new()), ty, span);
                }
                if args.len() > 1 && args.iter().all(|a| a.name.is_none()) {
                    let items: Vec<ast::Expr> = args.iter().map(|a| a.value.clone()).collect();
                    return self.list(&items, hint.filter(|t| matches!(t, Ty::List(_))), span);
                }
                let src = &args[0].value;
                let collect_directly = matches!(
                    &src.kind,
                    ast::ExprKind::Range { .. } | ast::ExprKind::Generic { .. }
                ) || matches!(
                    &src.kind,
                    ast::ExprKind::Field { name, .. }
                        if matches!(name.text.as_str(), "bounds" | "cuts" | "subsets")
                ) || matches!(
                    &src.kind,
                    ast::ExprKind::Name(n)
                        if self.lookup(n).is_none()
                            && (self.resolve_domain_name(n).is_some()
                                || matches!(self.get_sym(n), Some(Sym::Struct(sid)) if self.structs[sid].index_domain.is_some()))
                );
                if collect_directly {
                    return self.collect_iter_to_list(src, span);
                }
                let val = self.expr(src, None);
                match val.ty.clone() {
                    Ty::List(_) => val,
                    Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => self.mk(
                        ExprKind::Builtin(Builtin::ToList, vec![val]),
                        Ty::List(elem),
                        span,
                    ),
                    Ty::Str | Ty::StrView => self.mk(
                        ExprKind::Builtin(Builtin::StrToList, vec![val]),
                        Ty::list(Ty::Char),
                        span,
                    ),
                    Ty::Array { elem, .. } => self.mk(
                        ExprKind::Builtin(Builtin::ArrayToList, vec![val]),
                        Ty::List(elem),
                        span,
                    ),
                    Ty::Set(elem) | Ty::HashSet(elem) => self.mk(
                        ExprKind::Builtin(Builtin::Sorted, vec![val]),
                        Ty::List(elem),
                        span,
                    ),
                    Ty::DomainVal | Ty::Range(_) | Ty::Map(..) | Ty::HashMap(..) => {
                        self.collect_iter_to_list(src, span)
                    }
                    ref ty @ (Ty::Int | Ty::Bool | Ty::Char | Ty::Float) => {
                        self.not_contents(false, src, ty, span)
                    }
                    _ => val,
                }
            }
            "Set" | "HashSet" => {
                let hashed = name == "HashSet";
                if args.is_empty() {
                    let ty = match hint {
                        Some(ty @ (Ty::Set(_) | Ty::HashSet(_))) => ty.clone(),
                        _ if hashed => Ty::HashSet(Box::new(Ty::Int)),
                        _ => Ty::Set(Box::new(Ty::Int)),
                    };
                    return self.mk(ExprKind::Tuple(Vec::new()), ty, span);
                }
                let (iter, elem) = self.iter(&args[0].value);
                let ty = if hashed {
                    Ty::HashSet(Box::new(elem.clone()))
                } else {
                    Ty::Set(Box::new(elem.clone()))
                };
                self.do_expr(Some(&ty), span, |lw, out| {
                    let acc = lw.temp(ty.clone());
                    out.push(Stmt::Let(
                        acc,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
                    ));
                    let lid = lw.open_loop(None);
                    let slot = lw.temp(elem);
                    let item = lw.local(slot, span);
                    let ins = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Insert,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![item],
                        ),
                        Ty::Bool,
                        span,
                    );
                    lw.cx.loops.pop();
                    out.push(Stmt::For(
                        lid,
                        slot,
                        iter,
                        Block {
                            stmts: vec![Stmt::Expr(ins)],
                        },
                    ));
                    let res = lw.local(acc, span);
                    out.push(Stmt::Give(res));
                })
            }
            "Map" | "HashMap" => {
                let hashed = name == "HashMap";
                if let [src] = args {
                    let typed = matches!(hint, Some(Ty::HashMap(..)) if hashed)
                        || matches!(hint, Some(Ty::Map(..)) if !hashed);
                    let ty = hint.cloned().unwrap_or(Ty::Unit);
                    return self.map_from(hashed, ty, typed, &src.value, span);
                }
                let ty = match hint {
                    Some(ty @ (Ty::Map(..) | Ty::HashMap(..))) => ty.clone(),
                    _ if hashed => Ty::HashMap(Box::new(Ty::Int), Box::new(Ty::Int)),
                    _ => Ty::Map(Box::new(Ty::Int), Box::new(Ty::Int)),
                };
                self.mk(ExprKind::Tuple(Vec::new()), ty, span)
            }
            "Int" | "I128" | "U64" | "F64" | "Char" | "Str" | "Bool" => {
                if name == "Str" && args.is_empty() {
                    return self.mk(ExprKind::Str(Vec::new()), Ty::Str, span);
                }
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                self.convert(name, value, span)
            }
            _ => {
                let message = format!("no fn named `{name}` here");
                self.error(codes::UNRESOLVED_NAME, span, message);
                self.err_expr(span)
            }
        }
    }

    /// `sep.join(items)`, with `none` written as `none_text` when given.
    pub(in crate::lower) fn join(
        &mut self,
        sep: Expr,
        items: Expr,
        none_text: Option<&Expr>,
        span: Span,
    ) -> Expr {
        let items = match &items.ty {
            Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => {
                let list_ty = Ty::list((**e).clone());
                self.mk(
                    ExprKind::Builtin(Builtin::ToList, vec![items]),
                    list_ty,
                    span,
                )
            }
            Ty::Range(e) => {
                let elem = (**e).clone();
                self.collected(elem.clone(), span, |lw, out, push| {
                    let item = lw.temp(elem);
                    let lid = lw.fresh_loop();
                    let body = Block {
                        stmts: vec![push(lw, lw.local(item, span))],
                    };
                    out.push(Stmt::For(lid, item, Iter::RangeVal(items, false), body));
                })
            }
            _ => items,
        };
        let flat = match &items.ty {
            Ty::List(_) => true,
            Ty::Array { dims, .. } => dims.len() == 1,
            Ty::Error => return items,
            _ => false,
        };
        if !flat {
            let message = format!(
                "`join` takes a list or a 1-D array, not a `{}`",
                self.show(&items.ty)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            let (new, at) = (format!("List({})", self.source(span)), self.span(span));
            let text = self.show(&items.ty) == "Str";
            if let (true, Some(last)) = (text, self.diags.last_mut()) {
                last.help = Some(format!("a string's characters are the list `{new}`"));
                last.fix(format!("write `{new}`"), at, new);
            }
            return self.err_expr(span);
        }
        let mut args = vec![sep, items];
        args.extend(none_text.cloned());
        self.mk(ExprKind::Builtin(Builtin::Join, args), Ty::Str, span)
    }

    /// One `print` argument under `sep:` / `none:`, as the text it prints.
    pub(in crate::lower) fn print_piece(
        &mut self,
        value: Expr,
        sep: Option<&Expr>,
        none_text: Option<&Expr>,
        span: Span,
    ) -> Expr {
        let flat = match &value.ty {
            Ty::List(_) | Ty::ListView(..) | Ty::Keyed(..) | Ty::MutView(..) | Ty::Range(_) => true,
            Ty::Array { dims, .. } => dims.len() == 1,
            _ => false,
        };
        if flat {
            let sep = match sep {
                Some(sep) => sep.clone(),
                None => self.mk(ExprKind::Str(b" ".to_vec()), Ty::Str, span),
            };
            return self.join(sep, value, none_text, span);
        }
        let (Ty::Opt(inner), Some(none_text)) = (value.ty.clone(), none_text) else {
            return value;
        };
        let none_text = none_text.clone();
        self.do_expr(Some(&Ty::Str), span, |lw, out| {
            let slot = lw.temp(value.ty.clone());
            out.push(Stmt::Let(slot, Some(value)));
            let held = lw.local(slot, span);
            let some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
            let inside = lw.mk(ExprKind::Unwrap(Box::new(held), false), *inner, span);
            let shown = lw.mk(
                ExprKind::Builtin(Builtin::ShowStr, vec![inside]),
                Ty::Str,
                span,
            );
            let text = lw.mk(
                ExprKind::If(Box::new(some), Box::new(shown), Box::new(none_text)),
                Ty::Str,
                span,
            );
            out.push(Stmt::Give(text));
        })
    }

    pub(in crate::lower) fn convert(&mut self, to: &str, value: Expr, span: Span) -> Expr {
        let from = value.ty.clone();
        let (cast, ty) = match (to, &from) {
            (_, Ty::Error) => return value,
            ("Int", Ty::Int)
            | ("I128", Ty::I128)
            | ("U64", Ty::U64)
            | ("F64", Ty::Float)
            | ("Char", Ty::Char)
            | ("Str", Ty::Str)
            | ("Bool", Ty::Bool) => return value,
            ("Int", Ty::Float) => {
                self.error(
                    codes::TYPE_MISMATCH,
                    span,
                    "`Int` of an F64 does not say how to round",
                );
                // `Int(sqrt(n))`, C++'s `(int)sqrt(n)`: the exact root is `n.isqrt`.
                let root = match &value.kind {
                    ExprKind::Builtin(Builtin::Sqrt, args)
                        if matches!(
                            args.first().map(|a| &a.kind),
                            Some(ExprKind::Cast(Cast::IntToFloat, _))
                        ) =>
                    {
                        super::float_fn::sqrt_operand(&self.source(span))
                    }
                    _ => None,
                };
                let Some(n) = root else {
                    self.help("write `floor(x)`, `ceil(x)`, `round(x)` or `trunc(x)`");
                    return self.err_expr(span);
                };
                let simple = n.chars().all(|c| c.is_alphanumeric() || "_.".contains(c));
                let isqrt = if simple {
                    format!("{n}.isqrt")
                } else {
                    format!("({n}).isqrt")
                };
                self.help(format!(
                    "write `{isqrt}`, the exact integer square root; through `F64` it is off by one for some values past 2**52"
                ));
                let at = self.span(span);
                if let Some(last) = self.diags.last_mut() {
                    last.fixes.push(cpl_diag::Fix {
                        title: format!("write `{isqrt}`"),
                        edits: vec![(at, isqrt)],
                    });
                }
                return self.err_expr(span);
            }
            ("Int", Ty::Char) => (Cast::CharToInt, Ty::Int),
            ("Int", Ty::Str) => {
                return self.mk(
                    ExprKind::Builtin(Builtin::ParseInt, vec![value]),
                    Ty::Int,
                    span,
                );
            }
            ("Int", Ty::Pos(d)) => (Cast::Label(*d), Ty::Int),
            ("Int", Ty::Bound(_)) => (Cast::Ord, Ty::Int),
            ("Int", Ty::Enum(eid))
                if self.enums[*eid].variants.iter().all(|(_, f)| f.is_empty()) =>
            {
                (Cast::EnumVal(*eid), Ty::Int)
            }
            ("Int", Ty::Bool) => {
                let one = self.int_lit(1, span);
                let zero = self.int_lit(0, span);
                return self.mk(
                    ExprKind::If(Box::new(value), Box::new(one), Box::new(zero)),
                    Ty::Int,
                    span,
                );
            }
            ("Int", Ty::Enum(eid))
                if !self.enums[*eid].variants.is_empty()
                    && self.enums[*eid]
                        .variants
                        .iter()
                        .all(|(_, fs)| fs.is_empty()) =>
            {
                let eid = *eid;
                let loc = self.temp(Ty::Enum(eid));
                let init = Stmt::Let(loc, Some(value));
                let discs = self
                    .enum_discriminants
                    .get(eid)
                    .cloned()
                    .unwrap_or_default();
                let n = self.enums[eid].variants.len();
                let mut acc =
                    self.int_lit(discs.get(n - 1).copied().unwrap_or((n - 1) as i64), span);
                for tag in (0..n.saturating_sub(1)).rev() {
                    let disc = discs.get(tag).copied().unwrap_or(tag as i64);
                    let cond = self.mk(
                        ExprKind::IsVariant(
                            Box::new(self.mk(ExprKind::Local(loc), Ty::Enum(eid), span)),
                            tag,
                        ),
                        Ty::Bool,
                        span,
                    );
                    let then = self.int_lit(disc, span);
                    acc = self.mk(
                        ExprKind::If(Box::new(cond), Box::new(then), Box::new(acc)),
                        Ty::Int,
                        span,
                    );
                }
                return self.mk(
                    ExprKind::Do(Block {
                        stmts: vec![init, Stmt::Give(acc)],
                    }),
                    Ty::Int,
                    span,
                );
            }
            ("F64", Ty::Int) => (Cast::IntToFloat, Ty::Float),
            ("I128", Ty::Int) => (Cast::IntToI128, Ty::I128),
            ("Int", Ty::I128) => (Cast::I128ToInt, Ty::Int),
            ("U64", Ty::Int) => (Cast::IntToU64, Ty::U64),
            ("Int", Ty::U64) => (Cast::U64ToInt, Ty::Int),
            ("U64", Ty::I128) => (Cast::I128ToU64, Ty::U64),
            ("I128", Ty::U64) => (Cast::U64ToI128, Ty::I128),
            ("F64", Ty::I128) => (Cast::I128ToFloat, Ty::Float),
            ("F64", Ty::U64) => (Cast::U64ToFloat, Ty::Float),
            ("Char", Ty::Int) => (Cast::IntToChar, Ty::Char),
            // A position of a `Char` domain is its char (R04.2.24).
            ("Char", Ty::Pos(d)) if self.char_domains.contains(d) => {
                let d = *d;
                let code = self.cast(Cast::Label(d), value, Ty::Int);
                return self.cast(Cast::IntToChar, code, Ty::Char);
            }
            ("Str", Ty::List(elem)) if **elem == Ty::Char => {
                return self.mk(
                    ExprKind::Builtin(Builtin::ListToStr, vec![value]),
                    Ty::Str,
                    span,
                );
            }
            ("Str", _) => {
                return self.mk(
                    ExprKind::Builtin(Builtin::ShowStr, vec![value]),
                    Ty::Str,
                    span,
                );
            }
            ("Bool", Ty::Int) => {
                let zero = self.int_lit(0, span);
                return self.mk(
                    ExprKind::Cmp(CmpOp::Ne, Box::new(value), Box::new(zero)),
                    Ty::Bool,
                    span,
                );
            }
            _ => return self.convert_by_impl(to, value, span),
        };
        self.mk(ExprKind::Cast(cast, Box::new(value)), ty, span)
    }

    pub(in crate::lower) fn len_of(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Array { .. }
            | Ty::List(_)
            | Ty::ListView(_, _)
            | Ty::Keyed(_, _)
            | Ty::MutView(..)
            | Ty::Str
            | Ty::StrView
            | Ty::Map(..)
            | Ty::HashMap(..)
            | Ty::Set(_)
            | Ty::HashSet(_)
            | Ty::Range(_) => self.mk(ExprKind::Builtin(Builtin::Len, vec![value]), Ty::Int, span),
            Ty::Error => value,
            _ => {
                let message = format!("a `{}` has no length", self.show(&value.ty));
                self.error(codes::UNKNOWN_FIELD, span, message);
                self.err_expr(span)
            }
        }
    }

    pub(in crate::lower) fn sorted(&mut self, value: Expr, span: Span) -> Expr {
        match &value.ty {
            Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                let ty = Ty::list((**elem).clone());
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![value]), ty, span)
            }
            Ty::Array { elem, .. } => {
                let elem = elem.clone();
                let list = self.mk(
                    ExprKind::Builtin(Builtin::ArrayToList, vec![value]),
                    Ty::List(elem.clone()),
                    span,
                );
                self.mk(
                    ExprKind::Builtin(Builtin::Sorted, vec![list]),
                    Ty::List(elem),
                    span,
                )
            }
            Ty::Set(elem) | Ty::HashSet(elem) => {
                // A set is walked in order already.
                let ty = Ty::List(elem.clone());
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![value]), ty, span)
            }
            // A range goes up already: its list is what it goes over.
            Ty::Range(elem) => {
                let elem = (**elem).clone();
                self.walked_list(Iter::RangeVal(value, false), elem, span)
            }
            ty if ty.is_seq() => {
                let elem = ty.seq_elem().unwrap();
                if !self.ordered(&elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(&elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                let list = self.walked_list(Iter::Seq(value, false), elem.clone(), span);
                self.mk(
                    ExprKind::Builtin(Builtin::Sorted, vec![list]),
                    Ty::list(elem),
                    span,
                )
            }
            Ty::Error => value,
            other => {
                let message = format!("`sorted` takes a list, not a `{}`", self.show(other));
                self.error(codes::TYPE_MISMATCH, span, message);
                self.err_expr(span)
            }
        }
    }

    /// Every position of a domain, first to last, as a list.
    pub(in crate::lower) fn positions(&mut self, d: DomainId, span: Span) -> Expr {
        let list_ty = Ty::list(Ty::Pos(d));
        self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
            let all = lw.temp(list_ty.clone());
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push(Stmt::Let(all, Some(empty)));
            let at = lw.temp(Ty::Pos(d));
            let item = lw.local(at, span);
            let push = lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(all),
                        ty: list_ty.clone(),
                        span,
                    }),
                    vec![item],
                ),
                Ty::Unit,
                span,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                step: None,
                rev: false,
                closed: false,
            };
            out.push(Stmt::For(
                lid,
                at,
                iter,
                Block {
                    stmts: vec![Stmt::Expr(push)],
                },
            ));
            out.push(Stmt::Give(lw.local(all, span)));
        })
    }

    /// The list of what `lo..hi` goes over, in order.
    pub(in crate::lower) fn range_list(
        &mut self,
        lo: Expr,
        hi: Expr,
        elem: Ty,
        span: Span,
    ) -> Expr {
        self.walked_list(
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            },
            elem,
            span,
        )
    }

    /// The list of what `iter` goes over, in the order it goes.
    pub(in crate::lower) fn walked_list(&mut self, iter: Iter, elem: Ty, span: Span) -> Expr {
        self.collected(elem.clone(), span, |lw, out, push| {
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let at = lw.local(item, span);
            let body = Block {
                stmts: vec![push(lw, at)],
            };
            out.push(Stmt::For(lid, item, iter, body));
        })
    }

    /// `r.rev` kept as a value: the list of what the range goes over, last
    /// first. A loop over `r.rev` never comes here and builds no list.
    pub(in crate::lower) fn reversed_range(&mut self, range: Expr, span: Span) -> Expr {
        let Ty::Range(elem) = &range.ty else {
            unreachable!("only a range is asked for")
        };
        let elem = (**elem).clone();
        let iter = match range.kind {
            ExprKind::Range { lo, hi } => Iter::Range {
                lo: *lo,
                hi: *hi,
                step: None,
                rev: true,
                closed: false,
            },
            _ => Iter::RangeVal(range, true),
        };
        self.walked_list(iter, elem, span)
    }

    /// `sorted(xs, by: ...)`
    pub(in crate::lower) fn sorted_by(
        &mut self,
        value: Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        // `sorted(V, by: ...)`: the positions of a domain, in another order.
        let value = match (value.kind, &value.ty) {
            (ExprKind::DomainVal(d), _) => self.positions(d, span),
            // `sorted(D.all, by: ...)`, `sorted(0..n, by: ...)`: a range
            // written out is the list of what it goes over.
            (ExprKind::Range { lo, hi }, Ty::Range(of)) => {
                self.range_list(*lo, *hi, (**of).clone(), span)
            }
            (kind, Ty::Range(of)) => {
                let of = (**of).clone();
                let range = Expr { kind, ..value };
                self.walked_list(Iter::RangeVal(range, false), of, span)
            }
            (kind, _) => Expr { kind, ..value },
        };
        let list_ty = match value.ty.clone() {
            Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => Ty::List(e),
            Ty::Array { elem, .. } => Ty::List(elem),
            Ty::Set(e) | Ty::HashSet(e) => Ty::List(e),
            ty if ty.is_seq() => Ty::list(ty.seq_elem().unwrap()),
            Ty::Error => return value,
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    span,
                    format!("`sorted` takes a sequence, not a `{}`", self.show(&other)),
                );
                return self.err_expr(span);
            }
        };
        let elem = match &list_ty {
            Ty::List(e) => (**e).clone(),
            _ => unreachable!(),
        };
        self.do_expr(Some(&list_ty), span, |lw, out| {
            let init = match value.ty {
                Ty::List(_) => lw.mk(
                    ExprKind::Builtin(Builtin::Copy, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..) => lw.mk(
                    ExprKind::Builtin(Builtin::ToList, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                Ty::Array { .. } => lw.mk(
                    ExprKind::Builtin(Builtin::ArrayToList, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                Ty::Set(_) | Ty::HashSet(_) => lw.mk(
                    ExprKind::Builtin(Builtin::Sorted, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                _ => lw.walked_list(Iter::Seq(value, false), elem.clone(), span),
            };
            let slot = lw.temp(list_ty.clone());
            out.push(Stmt::Let(slot, Some(init)));
            let place = Place {
                kind: PlaceKind::Local(slot),
                ty: list_ty.clone(),
                span: lw.span(span),
            };
            let sort_expr = lw.sort_place(place, &elem, args, span);
            out.push(Stmt::Expr(sort_expr));
            let res = lw.local(slot, span);
            out.push(Stmt::Give(res));
        })
    }

    /// `runs(seq)` -> `List<(T, Int)>` grouping adjacent equal elements.
    pub(in crate::lower) fn lower_runs(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("runs", args, 1, span) else {
            return self.err_expr(span);
        };
        let seq = self.expr(exprs[0], None);
        let elem = match seq.ty.seq_elem() {
            Some(e) => e,
            None if seq.ty == Ty::Error => return self.err_expr(span),
            None => {
                self.error(
                    codes::TYPE_MISMATCH,
                    exprs[0].span,
                    format!("`runs` takes a sequence, not `{}`", self.show(&seq.ty)),
                );
                return self.err_expr(span);
            }
        };
        let pair_ty = Ty::Tuple(vec![elem.clone(), Ty::Int]);
        let res_ty = Ty::list(pair_ty.clone());
        self.do_expr(Some(&res_ty), span, |lw, out| {
            let acc = lw.temp(res_ty.clone());
            out.push(Stmt::Let(
                acc,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), res_ty.clone(), span)),
            ));
            let lid = lw.open_loop(None);
            let item_slot = lw.temp(elem.clone());
            let acc_expr = lw.local(acc, span);
            let acc_len = lw.mk(
                ExprKind::Builtin(Builtin::Len, vec![acc_expr.clone()]),
                Ty::Int,
                span,
            );
            let zero = lw.int_lit(0, span);
            let nonempty = lw.mk(
                ExprKind::Cmp(CmpOp::Gt, Box::new(acc_len.clone()), Box::new(zero)),
                Ty::Bool,
                span,
            );
            let last_idx = lw.int_add(acc_len, -1);
            let last_elem = lw.mk(
                ExprKind::Index(Box::new(acc_expr.clone()), vec![last_idx.clone()]),
                pair_ty.clone(),
                span,
            );
            let last_val = lw.mk(ExprKind::Field(Box::new(last_elem), 0), elem.clone(), span);
            let eq = lw.mk(
                ExprKind::Cmp(
                    CmpOp::Eq,
                    Box::new(last_val),
                    Box::new(lw.local(item_slot, span)),
                ),
                Ty::Bool,
                span,
            );
            let cond = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(nonempty), Box::new(eq)),
                Ty::Bool,
                span,
            );
            let count_place = Place {
                kind: PlaceKind::Field(
                    Box::new(Place {
                        kind: PlaceKind::Index(
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: res_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![last_idx],
                        ),
                        ty: pair_ty.clone(),
                        span: lw.span(span),
                    }),
                    1,
                ),
                ty: Ty::Int,
                span: lw.span(span),
            };
            let inc = Stmt::Assign(
                count_place,
                AssignOp::Binary(BinOp::Add),
                lw.int_lit(1, span),
            );
            let pair = lw.mk(
                ExprKind::Tuple(vec![lw.local(item_slot, span), lw.int_lit(1, span)]),
                pair_ty.clone(),
                span,
            );
            let push = Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(acc),
                        ty: res_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![pair],
                ),
                Ty::Unit,
                span,
            ));
            lw.cx.loops.pop();
            let seq_iter = if matches!(seq.ty, Ty::Range(_)) {
                Iter::RangeVal(seq, false)
            } else {
                Iter::Seq(seq, false)
            };
            out.push(Stmt::For(
                lid,
                item_slot,
                seq_iter,
                Block {
                    stmts: vec![Stmt::If(
                        cond,
                        Block { stmts: vec![inc] },
                        Block { stmts: vec![push] },
                    )],
                },
            ));
            let res = lw.local(acc, span);
            out.push(Stmt::Give(res));
        })
    }

    /// The spec of a type that says everything `read()` needs on its own.
    /// `read_output()` of a value laid out as `spec`: a `read()` from
    /// what the solution printed, then back to the input.
    pub(in crate::lower) fn read_printed(&mut self, spec: Spec, span: Span) -> Expr {
        let want = spec.ty();
        self.do_expr(Some(&want), span, |lw, out| {
            let swap = lw.mk(
                ExprKind::Builtin(Builtin::SwapInput, Vec::new()),
                Ty::Unit,
                span,
            );
            out.push(Stmt::Expr(swap.clone()));
            let printed = lw.read(spec, span);
            let slot = lw.temp(printed.ty.clone());
            out.push(Stmt::Let(slot, Some(printed)));
            out.push(Stmt::Expr(swap));
            out.push(Stmt::Give(lw.local(slot, span)));
        })
    }

    pub(in crate::lower) fn spec_of(&mut self, ty: &Ty) -> Option<Spec> {
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
                        Dim::Sym(d) => Some(DimSpec::Sym(*d)),
                        Dim::Enum(e) => Some(DimSpec::Enum(*e)),
                        Dim::Int | Dim::IntFrom(_) => None,
                    })
                    .collect();
                Spec::Array {
                    dims: dims?,
                    elem: Box::new(self.spec_of(elem)?),
                }
            }
            Ty::Opt(inner) => Spec::Opt(Box::new(self.spec_of(inner)?), None),
            Ty::List(elem) => Spec::List(Box::new(self.spec_of(elem)?)),
            // `l r`, the inclusive `l..=r`, checked `l <= r`. (R10.2.3)
            Ty::Range(inner) if **inner == Ty::Int => {
                let span = Span::default();
                let range = ty.clone();
                let call = self.do_expr(Some(&range), span, |lw, out| {
                    let mut end = || {
                        let slot = lw.temp(Ty::Int);
                        let read = lw.mk(
                            ExprKind::Read(Box::new(Spec::Plain(Ty::Int))),
                            Ty::Int,
                            span,
                        );
                        out.push(Stmt::Let(slot, Some(read)));
                        lw.local(slot, span)
                    };
                    let (l, r) = (end(), end());
                    let bad = lw.mk(
                        ExprKind::Cmp(CmpOp::Gt, Box::new(l.clone()), Box::new(r.clone())),
                        Ty::Bool,
                        span,
                    );
                    let trap = Stmt::Trap(
                        "a `Range` is read as `l r` with `l <= r`".into(),
                        lw.span(span),
                    );
                    out.push(Stmt::If(bad, Block { stmts: vec![trap] }, Block::default()));
                    let hi = lw.int_add(r, 1);
                    let value = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        range.clone(),
                        span,
                    );
                    out.push(Stmt::Give(value));
                });
                Spec::Call(Box::new(call))
            }
            other => Spec::Plain(other.clone()),
        })
    }

    /// `s.parse<T>()`: a `T` read from `s` by the rules of `read<T>()`.
    /// (R10.2.4)
    pub(in crate::lower) fn parse_str(
        &mut self,
        text: &ast::Expr,
        ty: &ast::Type,
        span: Span,
    ) -> Expr {
        let spec = self.resolve(ty, true);
        let text = self.expect(text, &Ty::Str);
        let read = self.read(spec, span);
        let ExprKind::Read(spec) = read.kind else {
            return read;
        };
        let opt = Ty::opt(read.ty);
        self.mk(
            ExprKind::Read(Box::new(Spec::Parse(Box::new(text), spec))),
            opt,
            span,
        )
    }

    /// `read()` of a value shaped like `spec`.
    /// `read_all<T>(until: f)`: reads `T`s until the input ends or `f` holds for one.
    pub(in crate::lower) fn read_all(
        &mut self,
        ty: &ast::Type,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let elem = self.resolve_ty(ty);
        let Some(spec) = self.spec_of(&elem) else {
            let message = format!(
                "`read_all()` can't tell how much a `{}` is",
                self.show(&elem)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        let test = Ty::Fn(vec![elem.clone()], Box::new(Ty::Bool));
        let until = match args {
            [] => None,
            [a] if a.name.as_ref().is_none_or(|n| n.text == "until") => match &a.value.kind {
                ast::ExprKind::Name(n) if n == "None" && self.lookup(n).is_none() => None,
                _ => Some(self.expect(&a.value, &test)),
            },
            _ => {
                self.error(
                    codes::UNKNOWN_ARG_NAME,
                    span,
                    "`read_all` takes only `until:`".to_string(),
                );
                return self.err_expr(span);
            }
        };
        let list = Ty::List(Box::new(elem.clone()));
        self.do_expr(Some(&list), span, |lw, out| {
            let acc = lw.temp(list.clone());
            out.push(Stmt::Let(
                acc,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), list.clone(), span)),
            ));
            let until = until.map(|f| {
                let slot = lw.temp(test.clone());
                out.push(Stmt::Let(slot, Some(f)));
                slot
            });
            let lid = lw.open_loop(None);
            let stop = || Block {
                stmts: vec![Stmt::Break(None)],
            };
            let done = lw.mk(
                ExprKind::Builtin(Builtin::AtEof, Vec::new()),
                Ty::Bool,
                span,
            );
            let mut body = vec![Stmt::If(done, stop(), Block { stmts: Vec::new() })];
            let value = lw.temp(elem.clone());
            let read = lw.read(spec, span);
            body.push(Stmt::Let(value, Some(read)));
            if let Some(f) = until {
                let call = ExprKind::CallExpr(
                    Box::new(lw.local(f, span)),
                    vec![Arg::Value(lw.local(value, span))],
                );
                let hit = lw.mk(call, Ty::Bool, span);
                body.push(Stmt::If(hit, stop(), Block { stmts: Vec::new() }));
            }
            let item = lw.local(value, span);
            let place = Place {
                kind: PlaceKind::Local(acc),
                ty: list.clone(),
                span: lw.span(span),
            };
            body.push(Stmt::Expr(lw.mk(
                ExprKind::Mutate(Mutation::Push, Box::new(place), vec![item]),
                Ty::Unit,
                span,
            )));
            lw.cx.loops.pop();
            out.push(Stmt::While(
                lid,
                lw.mk(ExprKind::Bool(true), Ty::Bool, span),
                Block { stmts: body },
            ));
            out.push(Stmt::Give(lw.local(acc, span)));
        })
    }

    /// A spec with each `Range<Int>` inside spelled out as its read of `l r`.
    pub(in crate::lower) fn readable(&mut self, spec: Spec) -> Spec {
        match spec {
            Spec::Plain(ty @ (Ty::Range(_) | Ty::List(_))) => match self.spec_of(&ty) {
                Some(Spec::List(each)) => self.readable(Spec::List(each)),
                spec => spec.unwrap_or(Spec::Plain(ty)),
            },
            Spec::List(each) => {
                let each = self.readable(*each);
                self.counted_list(each)
            }
            // `with io(base: 1)`: a counted domain's `1` is its first position.
            Spec::Plain(Ty::Pos(d)) if self.base_one() && self.counted.contains(&d) => {
                let span = Span::default();
                let call = self.do_expr(Some(&Ty::Pos(d)), span, |lw, out| {
                    let read = lw.mk(
                        ExprKind::Read(Box::new(Spec::Plain(Ty::Int))),
                        Ty::Int,
                        span,
                    );
                    let label = lw.int_add(read, -1);
                    let pos = lw.mk(
                        ExprKind::Cast(Cast::FromLabel(d), Box::new(label)),
                        Ty::Pos(d),
                        span,
                    );
                    out.push(Stmt::Give(pos));
                });
                Spec::Call(Box::new(call))
            }
            // A product position reads as its parts, as it prints. (R10.2.3)
            Spec::Plain(Ty::Pos(d)) if self.product_domains.contains_key(&d) => {
                Spec::Call(Box::new(self.product_read(d)))
            }
            // A payload enum that is a sum domain reads by its tag too. (R10.2.3)
            Spec::Plain(Ty::Pos(d)) if self.sum_enums.contains_key(&d) => {
                let id = self.sum_enums[&d].enum_id;
                match self.tagged_read(id, Ty::Pos(d)) {
                    Some(call) => Spec::Call(Box::new(call)),
                    None => Spec::Plain(Ty::Pos(d)),
                }
            }
            Spec::Plain(Ty::Enum(id)) => match self.enum_read(id) {
                Some(call) => Spec::Call(Box::new(call)),
                None => Spec::Plain(Ty::Enum(id)),
            },
            Spec::Fields(ty, fields) => {
                Spec::Fields(ty, fields.into_iter().map(|f| self.readable(f)).collect())
            }
            Spec::Array { dims, elem } => Spec::Array {
                dims,
                elem: Box::new(self.readable(*elem)),
            },
            Spec::Opt(inner, absent) => Spec::Opt(Box::new(self.readable(*inner)), absent),
            other => other,
        }
    }

    /// An enum read from its tag token, then that variant's fields. (R10.2.3)
    pub(in crate::lower) fn enum_read(&mut self, id: EnumId) -> Option<Expr> {
        self.tagged_read(id, Ty::Enum(id))
    }

    /// `id`'s variants read by tag, as a `ty`: the enum, or its sum domain.
    fn tagged_read(&mut self, id: EnumId, ty: Ty) -> Option<Expr> {
        let span = Span::default();
        let variants = self.enums[id].variants.clone();
        let mut fields = Vec::with_capacity(variants.len());
        for (_, tys) in &variants {
            let specs: Option<Vec<Spec>> = tys.iter().map(|ty| self.spec_of(ty)).collect();
            fields.push(specs?);
        }
        let tokens: Vec<String> = (0..variants.len())
            .map(|tag| self.enums[id].token(tag).to_string())
            .collect();
        let name = self.enums[id].name.clone();
        let sum = match ty {
            Ty::Pos(d) => Some(d),
            _ => None,
        };
        // Tags of one letter each read as a `Char`, so a column of them
        // reads packed (`WLW`) or spaced (`W L W`), as `[V]Char` does.
        let letters = tokens
            .iter()
            .all(|t| t.len() == 1 && !t.as_bytes()[0].is_ascii_digit());
        let tok_ty = if letters { Ty::Char } else { Ty::Str };
        Some(self.do_expr(Some(&ty), span, |lw, out| {
            let token = lw.temp(tok_ty.clone());
            let read = lw.mk(
                ExprKind::Read(Box::new(Spec::Plain(tok_ty.clone()))),
                tok_ty.clone(),
                span,
            );
            out.push(Stmt::Let(token, Some(read)));
            for (tag, (_, specs)) in variants.iter().zip(fields).enumerate() {
                let tags = [tokens[tag].clone()];
                let mut cond: Option<Expr> = None;
                for text in tags {
                    let lit = match letters {
                        true => lw.mk(ExprKind::Char(text.as_bytes()[0]), Ty::Char, span),
                        false => lw.mk(ExprKind::Str(text.into_bytes()), Ty::Str, span),
                    };
                    let tok = lw.local(token, span);
                    let eq = lw.mk(
                        ExprKind::Cmp(CmpOp::Eq, Box::new(tok), Box::new(lit)),
                        Ty::Bool,
                        span,
                    );
                    cond = Some(match cond {
                        Some(prev) => lw.mk(
                            ExprKind::Binary(BinOp::Or, Box::new(prev), Box::new(eq)),
                            Ty::Bool,
                            span,
                        ),
                        None => eq,
                    });
                }
                let mut body = Vec::new();
                let mut values = Vec::with_capacity(specs.len());
                for spec in specs {
                    let value = lw.read(spec, span);
                    let slot = lw.temp(value.ty.clone());
                    body.push(Stmt::Let(slot, Some(value)));
                    values.push(lw.local(slot, span));
                }
                let value = match sum {
                    Some(d) => lw.encode_sum_variant(d, tag, values, span),
                    None => lw.mk(ExprKind::Variant(tag, values), ty.clone(), span),
                };
                body.push(Stmt::Give(value));
                let cond = cond.expect("a variant has a name");
                out.push(Stmt::If(cond, Block { stmts: body }, Block::default()));
            }
            out.push(Stmt::Trap(
                format!("expected a `{name}` variant in the input"),
                lw.span(span),
            ));
        }))
    }

    pub(in crate::lower) fn read_with_args(
        &mut self,
        mut spec: Spec,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if !args.iter().all(|a| {
            a.name
                .as_ref()
                .is_some_and(|n| matches!(n.text.as_str(), "gen" | "none" | "base"))
        }) && self.positional_any("read", args, 0, span).is_none()
        {
            return self.err_expr(span);
        }
        if let Some(absent) = args
            .iter()
            .find(|a| a.name.as_ref().is_some_and(|n| n.text == "none"))
            .map(|a| self.expect(&a.value, &Ty::Int))
        {
            let want = spec.ty();
            if !reads_none_as(&mut spec, &absent) {
                let message = format!(
                    "`none:` is for reading a `?T`, and a `{}` has none",
                    self.show(&want)
                );
                self.error(codes::UNKNOWN_ARG_NAME, span, message);
            }
        }
        let base_arg = args
            .iter()
            .find(|a| a.name.as_ref().is_some_and(|n| n.text == "base"));
        let saved_ctx = self.contexts.len();
        let saved_base = self.io_base;
        if let Some(base) = base_arg {
            self.contexts.push((
                self.cur_mod,
                "io".to_string(),
                "base".to_string(),
                base.value.clone(),
            ));
            if let ast::ExprKind::Int(n) = base.value.kind {
                self.io_base = Some(n as i64);
            }
        }
        let out = self.read(spec, span);
        self.contexts.truncate(saved_ctx);
        self.io_base = saved_base;
        out
    }

    /// A value to print. Everything prints except nothing.
    pub(in crate::lower) fn fmt_arg(
        &mut self,
        value: Expr,
        digits: u32,
        span: Span,
    ) -> Option<FmtArg> {
        if value.ty == Ty::Unit {
            self.no_value_to_print(span);
            return None;
        }
        let value = self.in_base(value, span);
        let value = match value.ty {
            Ty::Pos(d) if self.product_domains.contains_key(&d) => {
                self.product_parts(d, value, span)
            }
            _ => value,
        };
        Some(FmtArg { value, digits })
    }

    /// A product position shows as the tuple of its parts: `D((1, 2))` is `1 2`.
    fn product_parts(&mut self, d: DomainId, value: Expr, span: Span) -> Expr {
        let factors = self.product_domains[&d].clone();
        let ty = Ty::Tuple(
            factors
                .iter()
                .map(|(_, f)| self.product_factor_ty(f))
                .collect(),
        );
        self.do_expr(Some(&ty.clone()), span, |lw, out| {
            let held = lw.hold(value, out);
            let parts = (0..factors.len())
                .map(|i| lw.decode_product_field(d, held.clone(), i, span))
                .collect();
            let tuple = lw.mk(ExprKind::Tuple(parts), ty, span);
            out.push(Stmt::Give(tuple));
        })
    }
}
