//! Built-in operations and the fused bitset folds.

use super::*;

impl<'p> Emitter<'p> {
    pub(super) fn builtin(
        &mut self,
        e: &Expr,
        op: Builtin,
        args: &[Expr],
        codes: Vec<Code>,
    ) -> Code {
        let text = |i: usize| codes.get(i).map(|c| c.text.clone()).unwrap_or_default();
        let first_ty = args.first().map(|a| &a.ty);
        let is_grid = matches!(first_ty, Some(Ty::Array { dims, .. }) if dims.len() > 1);
        let helper = |this: &mut Self, name: &'static str| {
            this.need(name);
            Code::primary(format!("cpl::{name}({})", text(0)))
        };
        match op {
            Builtin::Min | Builtin::Max => {
                let name = if op == Builtin::Min { "min" } else { "max" };
                let ty = self.ty(&e.ty);
                let all: Vec<String> = codes.iter().map(|c| c.text.clone()).collect();
                if all.len() == 2 {
                    Code::primary(format!("std::{name}<{ty}>({})", all.join(", ")))
                } else {
                    Code::primary(format!("std::{name}<{ty}>({{{}}})", all.join(", ")))
                }
            }
            Builtin::WrapAdd | Builtin::WrapSub | Builtin::WrapMul => {
                let sym = match op {
                    Builtin::WrapAdd => "+",
                    Builtin::WrapSub => "-",
                    _ => "*",
                };
                let int = self.ty(&Ty::Int);
                Code::primary(format!(
                    "{int}(uint64_t({}) {sym} uint64_t({}))",
                    text(0),
                    text(1)
                ))
            }
            Builtin::Abs if e.ty == Ty::Float => Code::primary(format!("std::fabs({})", text(0))),
            Builtin::Abs if e.ty == Ty::I128 => {
                self.need("abs128");
                Code::primary(format!("cpl::abs128({})", text(0)))
            }
            Builtin::Abs => Code::primary(format!("std::abs({})", widen(&codes[0]).text)),
            Builtin::Sqrt => {
                self.need("fsqrt");
                Code::primary(format!("cpl::fsqrt({})", text(0)))
            }
            Builtin::Flush => {
                self.need("out");
                Code::primary("(cpl::flush(), fflush(stdout))".to_string())
            }
            Builtin::Atan2 => Code::primary(format!("std::atan2({}, {})", text(0), text(1))),
            Builtin::Floor | Builtin::Ceil | Builtin::Round => {
                self.need("narrow");
                let f = match op {
                    Builtin::Floor => "floor",
                    Builtin::Ceil => "ceil",
                    _ => "round",
                };
                Code::primary(format!("cpl::to_int(std::{f}({}))", text(0)))
            }
            Builtin::CountQuery => {
                self.need("count_query");
                Code::primary(format!("cpl::count_query({})", text(0)))
            }
            // An `@interactor` runs in the interpreter, never as C++.
            Builtin::SwapInput => {
                self.need("fail");
                Code::primary("cpl::fail(\"an `@interactor` is not built to C++\")".to_string())
            }
            Builtin::AtEof => {
                self.need("at_eof");
                Code::primary("cpl::at_eof()".to_string())
            }
            Builtin::Popcount => {
                self.need("bits");
                Code::primary(format!("cpl::popcount({})", text(0)))
            }
            Builtin::Ctz => {
                self.need("bits");
                Code::primary(format!("cpl::ctz({})", text(0)))
            }
            Builtin::Clz => {
                self.need("bits");
                Code::primary(format!("cpl::clz({})", text(0)))
            }
            Builtin::AxisLen => Code::primary(format!("{}.n[{}]", codes[0].at(PRIMARY), text(1))),
            Builtin::BitWidth => {
                self.need("bits");
                Code::primary(format!("cpl::bit_width({})", text(0)))
            }
            Builtin::BitCeil => {
                self.need("bits");
                Code::primary(format!("cpl::bit_ceil({})", text(0)))
            }
            Builtin::Rand => {
                self.need("rng");
                Code::primary(format!("cpl::rand_range({}, {})", text(0), text(1)))
            }
            Builtin::Copy => {
                let l = args.first().and_then(|a| self.expr_elem_layout(a));
                Code::primary(format!(
                    "{}({})",
                    self.ty_with_elem_layout(&e.ty, l),
                    text(0)
                ))
            }
            Builtin::ListFilled => {
                let lty = self.ty(&e.ty);
                Code::primary(format!("{lty}((size_t)({}), {})", text(0), text(1)))
            }
            Builtin::Join => {
                let elem = match &args[1].ty {
                    Ty::List(elem)
                    | Ty::ListView(elem, _)
                    | Ty::Keyed(elem, _)
                    | Ty::MutView(elem)
                    | Ty::Array { elem, .. } => (**elem).clone(),
                    other => other.clone(),
                };
                let (sep, seq) = (codes[0].text.clone(), codes[1].text.clone());
                let none_text = codes.get(2).map(|c| c.text.clone());
                self.lambda("std::string", |this, out| {
                    this.need("capture");
                    let cap = this.temp();
                    let lead = pad(this.indent);
                    out.push_str(&format!("{lead}cpl::Capture {cap};\n"));
                    out.push_str(&format!("{lead}const auto& sep = {sep};\n"));
                    out.push_str(&format!("{lead}bool first = true;\n"));
                    out.push_str(&format!("{lead}for (const auto& item : {seq}) {{\n"));
                    out.push_str(&format!(
                        "{lead}    if (!first) cpl::put_str(sep);\n{lead}    first = false;\n"
                    ));
                    let item = Code::primary("item".to_string());
                    let stmt = match (&elem, &none_text) {
                        (Ty::Opt(inner), Some(text)) => {
                            let some =
                                this.show_stmt(&Code::new("*item", UNARY), inner, FLOAT_DIGITS);
                            format!("if (item) {{ {some} }} else cpl::put_str({text});")
                        }
                        _ => this.show_stmt(&item, &elem, FLOAT_DIGITS),
                    };
                    out.push_str(&format!("{lead}    {stmt}\n{lead}}}\n"));
                    out.push_str(&format!("{lead}return {cap}.take();\n"));
                })
            }
            Builtin::SeqGet if matches!(first_ty, Some(Ty::Range(_))) => {
                Code::primary(format!("{}.get({})", codes[0].at(PRIMARY), text(1)))
            }
            Builtin::SeqGet => {
                let opt = self.ty(&e.ty);
                let (seq, at) = (text(0), text(1));
                let elem = if matches!(first_ty, Some(Ty::Str | Ty::StrView)) {
                    "(cpl::u8)_seq[_idx]"
                } else {
                    "_seq[_idx]"
                };
                Code::primary(format!(
                    "([&]() -> {opt} {{ const auto& _seq = {seq}; ll _idx = {at}; return _idx < 0 || _idx >= (ll)_seq.size() ? {opt}() : {opt}({elem}); }}())"
                ))
            }
            Builtin::ShowStr => {
                let arg = &args[0];
                let code = codes[0].clone();
                self.lambda("std::string", |this, out| {
                    this.need("capture");
                    let cap = this.temp();
                    out.push_str(&format!("{}cpl::Capture {cap};\n", pad(this.indent)));
                    let stmt = this.show_stmt(&code, &arg.ty, FLOAT_DIGITS);
                    out.push_str(&format!("{}{stmt}\n", pad(this.indent)));
                    out.push_str(&format!("{}return {cap}.take();\n", pad(this.indent)));
                })
            }
            Builtin::ParseInt => {
                self.need("str_conv");
                Code::primary(format!("cpl::parse_int({})", text(0)))
            }
            Builtin::StrToList => {
                self.need("str_conv");
                Code::primary(format!("cpl::str_to_list({})", text(0)))
            }
            Builtin::ListToStr => {
                self.need("str_conv");
                Code::primary(format!("cpl::list_to_str({})", text(0)))
            }
            Builtin::ArrayToList => {
                if is_grid {
                    Code::primary(format!("{}.d", codes[0].at(PRIMARY)))
                } else {
                    codes[0].clone()
                }
            }
            Builtin::ListToArray => codes[0].clone(),
            Builtin::Len if is_grid => {
                Code::new(format!("(ll){}.d.size()", codes[0].at(PRIMARY)), UNARY)
            }
            Builtin::Len if matches!(first_ty, Some(Ty::Range(_))) => {
                Code::primary(format!("{}.len()", codes[0].at(PRIMARY)))
            }
            Builtin::Len if matches!(first_ty, Some(Ty::DomainVal)) => {
                Code::primary(format!("{}.len", codes[0].at(PRIMARY)))
            }
            Builtin::Len if self.is_row(&args[0]) => {
                let row = codes[0].at(PRIMARY);
                Code::new(
                    format!("(ll){}.size()", row.strip_suffix(".vec()").unwrap_or(&row)),
                    UNARY,
                )
            }
            Builtin::Len => Code::new(format!("(ll){}.size()", codes[0].at(PRIMARY)), UNARY),
            Builtin::First if matches!(first_ty, Some(Ty::Range(_))) => {
                Code::primary(format!("{}.first()", codes[0].at(PRIMARY)))
            }
            Builtin::Last if matches!(first_ty, Some(Ty::Range(_))) => {
                Code::primary(format!("{}.last()", codes[0].at(PRIMARY)))
            }
            Builtin::First => helper(self, "first"),
            Builtin::Last => helper(self, "last"),
            Builtin::FirstVal if matches!(first_ty, Some(Ty::Range(_))) => {
                self.need("fail");
                let r = text(0);
                Code::primary(format!(
                    "([&]() {{ auto r = {r}; if (r.empty()) cpl::fail(\"the range is empty\"); return r.lo; }}())"
                ))
            }
            Builtin::LastVal if matches!(first_ty, Some(Ty::Range(_))) => {
                self.need("fail");
                let r = text(0);
                Code::primary(format!(
                    "([&]() {{ auto r = {r}; if (r.empty()) cpl::fail(\"the range is empty\"); return (decltype(r.lo))(r.hi - 1); }}())"
                ))
            }
            Builtin::FirstVal => helper(self, "first_val"),
            Builtin::LastVal => helper(self, "last_val"),
            Builtin::Contains => {
                let holder = codes[0].at(PRIMARY);
                match first_ty {
                    Some(Ty::Range(_)) => Code::primary(format!("{holder}.has({})", text(1))),
                    Some(Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..)) => {
                        Code::new(format!("{holder}.count({}) != 0", text(1)), EQ)
                    }
                    Some(Ty::Str) if args[1].ty == Ty::Str => Code::new(
                        format!("{holder}.find({}) != std::string::npos", text(1)),
                        EQ,
                    ),
                    Some(Ty::Str) if args[1].ty == Ty::StrView => {
                        self.need("str_view");
                        Code::primary(format!("cpl::StrView({holder}).contains_str({})", text(1)))
                    }
                    Some(Ty::Str) => Code::new(
                        format!(
                            "{holder}.find((char){}) != std::string::npos",
                            codes[1].at(UNARY)
                        ),
                        EQ,
                    ),
                    Some(Ty::StrView) if matches!(args[1].ty, Ty::Str | Ty::StrView) => {
                        self.need("str_view");
                        Code::primary(format!("{holder}.contains_str(cpl::StrView({}))", text(1)))
                    }
                    Some(Ty::StrView) => {
                        self.need("str_view");
                        Code::primary(format!("{holder}.contains_char({})", text(1)))
                    }
                    _ => {
                        self.need("has");
                        let items = if is_grid {
                            format!("{holder}.d")
                        } else {
                            holder
                        };
                        Code::primary(format!("cpl::has({items}, {})", text(1)))
                    }
                }
            }
            Builtin::MapGet => {
                self.need("map_get_any");
                Code::primary(format!("cpl::map_get({}, {})", text(0), text(1)))
            }
            Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt => {
                self.need("ordered_near");
                let name = match op {
                    Builtin::FirstGe => "first_ge",
                    Builtin::FirstGt => "first_gt",
                    Builtin::LastLe => "last_le",
                    _ => "last_lt",
                };
                Code::primary(format!("cpl::{name}({}, {})", text(0), text(1)))
            }
            Builtin::SetMin | Builtin::SetMax if matches!(args[0].ty, Ty::Map(..)) => {
                self.need("ordered_near");
                let f = if op == Builtin::SetMin {
                    "first_of"
                } else {
                    "last_of"
                };
                Code::primary(format!("cpl::{f}({})", text(0)))
            }
            Builtin::SetMin => helper(self, "set_min"),
            Builtin::SetMax => helper(self, "set_max"),
            Builtin::Sorted => helper(self, "sorted"),
            Builtin::Reach => {
                self.need("reach");
                Code::primary(format!("cpl::reach({}, {}, {})", text(0), text(1), text(2)))
            }
            Builtin::Reversed => match first_ty {
                Some(Ty::Str | Ty::StrView) => {
                    self.need("str_view");
                    Code::primary(format!("cpl::str_rev({})", text(0)))
                }
                Some(Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(_)) => {
                    self.need("list_view");
                    Code::primary(format!("cpl::list_rev({})", text(0)))
                }
                Some(Ty::Array { dims, .. }) if dims.len() == 1 => {
                    self.need("list_view");
                    Code::primary(format!("cpl::list_rev({})", text(0)))
                }
                _ => helper(self, "reversed"),
            },
            Builtin::StrSlice => {
                self.need("str_view");
                Code::primary(format!(
                    "cpl::str_slice({}, {}, {})",
                    text(0),
                    text(1),
                    text(2)
                ))
            }
            Builtin::ListSlice => {
                self.need("list_view");
                Code::primary(format!(
                    "cpl::list_slice({}, {}, {})",
                    text(0),
                    text(1),
                    text(2)
                ))
            }
            Builtin::KeyLo
            | Builtin::KeyPos
            | Builtin::PosKey
            | Builtin::KeySlice
            | Builtin::Rekey => {
                self.need(if matches!(args[0].ty, Ty::StrView) {
                    "str_view"
                } else {
                    "list_view"
                });
                let name = match op {
                    Builtin::KeyLo => "key_lo",
                    Builtin::KeyPos => "key_pos",
                    Builtin::PosKey => "pos_key",
                    Builtin::Rekey if matches!(e.ty, Ty::Keyed(..)) => "rekey_own",
                    Builtin::Rekey => "rekey",
                    _ => "key_slice",
                };
                if matches!(op, Builtin::Rekey) {
                    self.need("rekey");
                }
                let all: Vec<String> = (0..args.len()).map(|i| text(i).to_string()).collect();
                Code::primary(format!("cpl::{name}({})", all.join(", ")))
            }
            Builtin::ToStr => {
                self.need("str_view");
                Code::primary(format!("cpl::to_str({})", text(0)))
            }
            Builtin::ToList => {
                self.need("list_view");
                Code::primary(format!("cpl::to_list({})", text(0)))
            }
            Builtin::StrConcat => {
                self.need("str_view");
                Code::primary(format!(
                    "cpl::str_concat(cpl::StrView({}), cpl::StrView({}))",
                    text(0),
                    text(1)
                ))
            }
            Builtin::ListConcat => {
                self.need("list_view");
                Code::primary(format!("cpl::list_concat({}, {})", text(0), text(1)))
            }
            Builtin::PosNext => {
                self.need("next_pos");
                Code::primary(format!("cpl::next_pos({}, {})", text(0), text(1)))
            }
            Builtin::PosPrev => helper(self, "prev_pos"),
            Builtin::PosTryOrd(d) => {
                self.need("pos_ops");
                let dn = ident(&self.program.domains[d]);
                Code::primary(format!("cpl::pos_try_ord({}, {dn}_len)", text(0)))
            }
            Builtin::SymIdx => {
                self.need("sym_idx");
                Code::primary(format!("cpl::sym_idx({}, {})", text(0), text(1)))
            }
            Builtin::PosOffset(d) => {
                self.need("pos_ops");
                let dn = ident(&self.program.domains[d]);
                Code::primary(format!(
                    "cpl::pos_offset({}, {}, {dn}_len)",
                    text(0),
                    text(1)
                ))
            }
            Builtin::PosCyc(d) => {
                self.need("pos_ops");
                let dn = ident(&self.program.domains[d]);
                Code::primary(format!("cpl::pos_cyc({}, {}, {dn}_len)", text(0), text(1)))
            }
            Builtin::BoundStepL(d) => {
                self.need("pos_ops");
                let dn = ident(&self.program.domains[d]);
                Code::primary(format!(
                    "cpl::bound_step((ll)({}) + 1 - ({}), {dn}_len)",
                    text(0),
                    text(1)
                ))
            }
            Builtin::BoundStepR(d) => {
                self.need("pos_ops");
                let dn = ident(&self.program.domains[d]);
                Code::primary(format!(
                    "cpl::bound_step((ll)({}) + ({}), {dn}_len)",
                    text(0),
                    text(1)
                ))
            }
            Builtin::BoundLeft(_d) => {
                self.need("pos_ops");
                Code::primary(format!("cpl::bound_left({})", text(0)))
            }
            Builtin::BoundRight(d) => {
                self.need("pos_ops");
                let dn = ident(&self.program.domains[d]);
                Code::primary(format!("cpl::bound_right({}, {dn}_len)", text(0)))
            }
            // `(a..b).lo` with `a` a literal at least 0 is `a`: `b` can neither trap nor
            // change anything, and the span cannot overflow, so `make` checks nothing.
            Builtin::RangeLo => match &args[0].kind {
                ExprKind::Range { lo, hi }
                    if matches!(lo.kind, ExprKind::Int(v) if v >= 0) && inert(hi) =>
                {
                    self.expr(lo)
                }
                _ => Code::primary(format!("{}.lo", codes[0].at(PRIMARY))),
            },
            Builtin::RangeHi => Code::primary(format!("{}.hi", codes[0].at(PRIMARY))),
            Builtin::RangeRev => Code::primary(format!("{}.rev()", codes[0].at(PRIMARY))),
            Builtin::RangeSum => {
                self.need("range_sum");
                if codes.len() == 2 {
                    Code::primary(format!("cpl::range_sum({}, {})", text(0), text(1)))
                } else {
                    Code::primary(format!("cpl::range_sum({})", text(0)))
                }
            }
        }
    }

    /// Fuse `(a & b & ...).len` / `.count` / `.is_empty` on `Bitset<D>` into a single
    /// zero-allocation word-wise loop over `U64` words.
    pub(super) fn try_fused_bitset_op(
        &mut self,
        e: &Expr,
        id: usize,
        args: &[Arg],
    ) -> Option<Code> {
        enum BitsetFold {
            Len,
            IsEmpty,
        }
        enum BitsetTree<'a> {
            Leaf(usize),
            Bin(&'static str, Box<BitsetTree<'a>>, Box<BitsetTree<'a>>),
            _Phantom(std::marker::PhantomData<&'a ()>),
        }

        if args.len() != 1 {
            return None;
        }
        let Arg::Value(inner) = &args[0] else {
            return None;
        };
        let fname = fn_name(&self.program.fns[id].name);
        let fold = if fname.ends_with("_len") || fname.ends_with("_count") {
            BitsetFold::Len
        } else if fname.ends_with("_is_empty") {
            BitsetFold::IsEmpty
        } else {
            return None;
        };
        if !is_bitset_ty(&inner.ty, self.program) {
            return None;
        }

        fn parse_tree<'a>(
            expr: &'a Expr,
            target_ty: &Ty,
            program: &cpl_ir::Program,
            leaves: &mut Vec<&'a Expr>,
            has_op: &mut bool,
        ) -> BitsetTree<'a> {
            if let ExprKind::Call(cid, cargs) = &expr.kind {
                if cargs.len() == 2 {
                    if let (Arg::Value(a), Arg::Value(b)) = (&cargs[0], &cargs[1]) {
                        if &a.ty == target_ty && &b.ty == target_ty {
                            let cname = fn_name(&program.fns[*cid].name);
                            let op = if cname.ends_with("_bitand") {
                                Some("&")
                            } else if cname.ends_with("_bitor") {
                                Some("|")
                            } else if cname.ends_with("_bitxor") {
                                Some("^")
                            } else {
                                None
                            };
                            if let Some(op) = op {
                                *has_op = true;
                                let l = parse_tree(a, target_ty, program, leaves, has_op);
                                let r = parse_tree(b, target_ty, program, leaves, has_op);
                                return BitsetTree::Bin(op, Box::new(l), Box::new(r));
                            }
                        }
                    }
                }
            }
            let idx = leaves.len();
            leaves.push(expr);
            BitsetTree::Leaf(idx)
        }

        fn emit_word_expr(tree: &BitsetTree<'_>, vars: &[String], wi: &str) -> String {
            match tree {
                BitsetTree::Leaf(i) => format!("{}.w[{wi}]", vars[*i]),
                BitsetTree::Bin(op, l, r) => format!(
                    "({} {op} {})",
                    emit_word_expr(l, vars, wi),
                    emit_word_expr(r, vars, wi)
                ),
                BitsetTree::_Phantom(_) => unreachable!(),
            }
        }

        let mut leaves = Vec::new();
        let mut has_op = false;
        let tree = parse_tree(inner, &inner.ty, self.program, &mut leaves, &mut has_op);
        if !has_op {
            return None;
        }

        let ret = self.ty(&e.ty);
        Some(self.lambda(&ret, |this, out| {
            let p = pad(this.indent);
            let mut vars = Vec::with_capacity(leaves.len());
            for leaf in &leaves {
                let code = this.expr(leaf).text;
                let v = this.temp();
                out.push_str(&format!("{p}const auto& {v} = {code};\n"));
                vars.push(v);
            }
            let wi = this.temp();
            let word_expr = emit_word_expr(&tree, &vars, &wi);
            match fold {
                BitsetFold::Len => {
                    let acc = this.temp();
                    out.push_str(&format!("{p}ll {acc} = 0;\n"));
                    out.push_str(&format!(
                        "{p}for (size_t {wi} = 0; {wi} < {}.w.size(); ++{wi}) {acc} += (ll)__builtin_popcountll({word_expr});\n",
                        vars[0]
                    ));
                    out.push_str(&format!("{p}return {acc};\n"));
                }
                BitsetFold::IsEmpty => {
                    out.push_str(&format!(
                        "{p}for (size_t {wi} = 0; {wi} < {}.w.size(); ++{wi}) if (({word_expr}) != 0ULL) return false;\n",
                        vars[0]
                    ));
                    out.push_str(&format!("{p}return true;\n"));
                }
            }
        }))
    }
}

/// `e` reads values and does nothing else: no call, no I/O, no trap.
fn inert(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Cast(_, x) => inert(x),
        ExprKind::Builtin(Builtin::Len, xs) => xs.iter().all(inert),
        ExprKind::Binary(cpl_ir::BinOp::Add, a, b) => {
            matches!(a.kind, ExprKind::Int(0)) && inert(b)
                || matches!(b.kind, ExprKind::Int(0)) && inert(a)
        }
        _ => false,
    }
}
