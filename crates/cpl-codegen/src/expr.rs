//! Expressions as C++.

use crate::emit::{
    pad, Code, Emitter, Touch, ADD, AND, BIT, COND, EQ, MUL, OR, PRIMARY, REL, SHIFT, UNARY,
};
use crate::names::{elem_ty, fn_name, ident, is_plain_enum};
use crate::ranges::Sign;
use crate::walk;
const FLOAT_DIGITS: &str = "10";

use cpl_ir::{
    Arg, BinOp, Builtin, Cast, CmpOp, DimSpec, Expr, ExprKind, FmtPiece, Mutation, NumKind, Place,
    PlaceKind, Stmt, Ty, UnOp,
};

pub(crate) fn int_literal(v: i64) -> Code {
    if v == i64::MIN {
        return Code::primary("(-9223372036854775807LL - 1)");
    }
    let narrow = v.unsigned_abs() < (1 << 31);
    Code {
        text: if narrow {
            v.to_string()
        } else {
            format!("{v}LL")
        },
        prec: if v < 0 { UNARY } else { PRIMARY },
        narrow,
    }
}

fn float_literal(v: f64) -> Code {
    if v.is_nan() {
        Code::primary("NAN")
    } else if v.is_infinite() {
        Code::new(if v > 0.0 { "INFINITY" } else { "-INFINITY" }, UNARY)
    } else {
        Code::new(
            format!("{v:?}"),
            if v.is_sign_negative() { UNARY } else { PRIMARY },
        )
    }
}

fn char_literal(c: u8) -> Code {
    match c {
        b'\'' => Code::primary("'\\''"),
        b'\\' => Code::primary("'\\\\'"),
        b'\n' => Code::primary("'\\n'"),
        b'\t' => Code::primary("'\\t'"),
        b'\r' => Code::primary("'\\r'"),
        0x20..=0x7e => Code::primary(format!("'{}'", c as char)),
        _ => Code::new(format!("(cpl::u8){c}"), UNARY),
    }
}

/// The bytes as a C++ string literal, quotes included.
pub(crate) fn string_literal(bytes: &[u8]) -> String {
    let mut out = String::from("\"");
    let mut last = 0u8;
    for &b in bytes {
        match b {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            // `??x` could be read as a trigraph by an old compiler.
            b'?' if last == b'?' => out.push_str("\\?"),
            0x20..=0x7e => out.push(b as char),
            _ => out.push_str(&format!("\\{b:03o}")),
        }
        last = b;
    }
    out.push('"');
    out
}

/// An `int` expression made `long long`, so arithmetic on it has 64 bits.
fn widen(code: &Code) -> Code {
    if !code.narrow {
        return code.clone();
    }
    let text = if code.text.parse::<i64>().is_ok() {
        format!("{}LL", code.text)
    } else {
        format!("(ll){}", code.at(UNARY))
    };
    Code {
        text,
        prec: code.prec.min(UNARY),
        narrow: false,
    }
}

fn infix(l: &Code, sym: &str, r: &Code, prec: u8) -> Code {
    Code::new(format!("{} {sym} {}", l.at(prec), r.at(prec + 1)), prec)
}

/// `e` as a position and a small step from it, when it is one.
fn neighbour(e: &Expr) -> Option<(&Expr, char, i64)> {
    let ExprKind::Binary(op @ (BinOp::Add | BinOp::Sub), l, r) = &e.kind else {
        return None;
    };
    match (&l.kind, &r.kind) {
        (ExprKind::Cast(Cast::Ord, pos), ExprKind::Int(step)) if (0..=1 << 20).contains(step) => {
            Some((pos, if *op == BinOp::Add { '+' } else { '-' }, *step))
        }
        _ => None,
    }
}

impl<'p> Emitter<'p> {
    /// What evaluating `e` reads, writes and prints.
    pub fn touch(&self, e: &Expr) -> Touch {
        let mut touch = Touch::default();
        walk::deep_expr(e, &mut |inner| match &inner.kind {
            ExprKind::Local(slot) => {
                touch.reads.insert((false, *slot));
            }
            ExprKind::Global(id) => {
                touch.reads.insert((true, *id));
            }
            ExprKind::Read(_)
            | ExprKind::Print(..)
            | ExprKind::Builtin(Builtin::CountQuery | Builtin::SwapInput, _) => touch.io = true,
            ExprKind::CallExpr(_, args) => {
                touch.io = true;
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        touch.writes.insert(walk::root(place));
                    }
                }
            }
            ExprKind::Call(id, args) => {
                let summary = &self.summaries[*id];
                touch.io |= summary.io;
                touch.reads.extend(summary.reads.iter().map(|g| (true, *g)));
                touch
                    .writes
                    .extend(summary.writes.iter().map(|g| (true, *g)));
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        touch.writes.insert(walk::root(place));
                    }
                }
            }
            ExprKind::MutSlice(place, _, _)
            | ExprKind::Mutate(_, place, _)
            | ExprKind::Sort(place, _, _) => {
                touch.writes.insert(walk::root(place));
            }
            ExprKind::Do(block) => walk::deep_stmts(block, &mut |stmt| match stmt {
                Stmt::Assign(place, ..) => {
                    touch.writes.insert(walk::root(place));
                }
                Stmt::Let(slot, _) | Stmt::For(_, slot, ..) => {
                    touch.writes.insert((false, *slot));
                }
                Stmt::DefDomain { .. } | Stmt::Assert { .. } | Stmt::Trap(..) => touch.io = true,
                _ => {}
            }),
            _ => {}
        });
        touch
    }

    /// A lambda called on the spot: `[&]() -> ret { body }()`.
    pub fn lambda(&mut self, ret: &str, body: impl FnOnce(&mut Self, &mut String)) -> Code {
        let mut text = format!("[&]() -> {ret} {{\n");
        self.indent += 1;
        body(self, &mut text);
        self.indent -= 1;
        text.push_str(&pad(self.indent));
        text.push_str("}()");
        Code::primary(text)
    }

    /// Emits `operands` and hands them to `build`. C++ does not say which
    /// operand of `a + b` or argument of `f(a, b)` runs first; the
    /// language says left to right. Where the difference could show, the
    /// operands are named in order first.
    pub fn sequence(
        &mut self,
        operands: &[&Expr],
        ret: &Ty,
        build: impl FnOnce(&mut Self, Vec<Code>) -> Code,
    ) -> Code {
        let touches: Vec<Touch> = operands.iter().map(|e| self.touch(e)).collect();
        let n = operands.len();
        let ordered = (0..n).any(|i| (i + 1..n).any(|j| touches[i].conflicts(&touches[j])));
        if !ordered {
            let codes = operands.iter().map(|e| self.expr(e)).collect();
            return build(self, codes);
        }
        let ret_text = self.ty(ret);
        let is_unit = *ret == Ty::Unit;
        self.lambda(&ret_text, |this, out| {
            let mut codes = Vec::with_capacity(n);
            for (i, e) in operands.iter().enumerate() {
                let code = this.expr(e);
                if i + 1 == n
                    || this.is_fixed(e)
                    || matches!(e.kind, ExprKind::Str(_) | ExprKind::None)
                {
                    codes.push(code);
                } else {
                    let name = this.temp();
                    out.push_str(&format!(
                        "{}{} {name} = {};\n",
                        pad(this.indent),
                        this.ty(&e.ty),
                        code.text
                    ));
                    codes.push(Code::primary(name));
                }
            }
            let result = build(this, codes);
            let lead = if is_unit { "" } else { "return " };
            out.push_str(&format!("{}{lead}{};\n", pad(this.indent), result.text));
        })
    }

    pub fn place(&mut self, place: &Place) -> String {
        self.place_at(place, false)
    }

    /// `store` is true for the target of a plain `=`, where a map makes
    /// room for a new key.
    pub fn place_at(&mut self, place: &Place, store: bool) -> String {
        if let Some(row) = self.cached_row(crate::rowcache::of_place(place)) {
            return row;
        }
        match &place.kind {
            PlaceKind::Local(slot) => self.names[*slot].clone(),
            PlaceKind::Global(id) => ident(&self.program.globals[*id].name),
            PlaceKind::Field(base, index) => {
                let b = self.place_at(base, false);
                match &base.ty {
                    Ty::Struct(id) => {
                        format!("{b}.{}", ident(&self.program.structs[*id].fields[*index].0))
                    }
                    _ => format!("std::get<{index}>({b})"),
                }
            }
            PlaceKind::Some(base) => format!("(*{})", self.place_at(base, false)),
            PlaceKind::Index(base, indices) => {
                let b = self.place_at(base, false);
                let touches: Vec<Touch> = indices.iter().map(|e| self.touch(e)).collect();
                let n = indices.len();
                if (0..n).any(|i| (i + 1..n).any(|j| touches[i].conflicts(&touches[j]))) {
                    // The arguments of a call run in no set order: name
                    // the indices one by one.
                    let cell = format!("{}&", elem_ty(self.program, self.niches, &place.ty));
                    return self
                        .lambda(&cell, |this, out| {
                            let mut at = Vec::with_capacity(n);
                            for index in indices {
                                let code = this.expr(index).text;
                                let name = this.temp();
                                out.push_str(&format!(
                                    "{}{} {name} = {code};\n",
                                    pad(this.indent),
                                    this.ty(&index.ty)
                                ));
                                at.push(name);
                            }
                            let call = if proven(indices) { "" } else { ".ck" };
                            out.push_str(&format!(
                                "{}return {b}{call}({});\n",
                                pad(this.indent),
                                at.join(", ")
                            ));
                        })
                        .text;
                }
                let at: Vec<String> = indices
                    .iter()
                    .map(|e| {
                        let t = self.expr(e).text;
                        if t.starts_with('[') {
                            format!("({t})")
                        } else {
                            t
                        }
                    })
                    .collect();
                match &base.ty {
                    Ty::Array { dims, .. } if dims.len() > 1 && proven(indices) => {
                        format!("{b}({})", at.join(", "))
                    }
                    Ty::Array { dims, .. } if dims.len() > 1 => {
                        format!("{b}.ck({})", at.join(", "))
                    }
                    Ty::Array { .. } | Ty::Str if !proven(indices) => {
                        if self.place_fixed_len(base).is_some() {
                            self.need("at_array");
                        } else {
                            self.need("at");
                        }
                        format!("cpl::at({b}, {})", at[0])
                    }
                    Ty::Map(..) | Ty::HashMap(..) if !store => format!("{b}.at({})", at.join(", ")),
                    Ty::List(_) if matches!(base.kind, PlaceKind::Global(id) if self.program.raw_lists.contains(&id)) =>
                    {
                        format!("{b}[{}]", at[0])
                    }
                    Ty::List(_) | Ty::MutView(_) if matches!(indices[0].kind, ExprKind::Cast(Cast::Raw, _)) => {
                        format!("{b}[{}]", at[0])
                    }
                    Ty::List(_) | Ty::MutView(_) if matches!((&base.kind, &indices[0].kind), (PlaceKind::Local(list), ExprKind::Local(i)) if self.inside.contains(&(*list, *i))) =>
                    {
                        format!("{b}[{}]", at[0])
                    }
                    Ty::List(_) | Ty::MutView(_) => {
                        self.need("at");
                        format!("cpl::at({b}, {})", at.join(", "))
                    }
                    _ => format!("{b}[{}]", at.join(", ")),
                }
            }
        }
    }

    /// True when writing `place` twice would run something twice.
    pub fn place_is_plain(&self, place: &Place) -> bool {
        let mut plain = true;
        walk::place_exprs(place, &mut |e| {
            let touch = self.touch(e);
            plain &= !touch.io && touch.writes.is_empty();
        });
        plain
    }

    /// An element of a `{...}` list, which C++ refuses to narrow: a
    /// `Char` or position that C++ computed as a wider type is cast back.
    fn braced(&mut self, e: &Expr) -> String {
        let code = self.expr(e);
        let typed = matches!(
            e.kind,
            ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Field(..)
                | ExprKind::Char(_)
                | ExprKind::Read(_)
        ) || matches!(
            (&e.kind, &e.ty),
            (ExprKind::Index(..), Ty::Pos(_) | Ty::Bound(_))
        );
        match &e.ty {
            Ty::Char | Ty::Pos(_) | Ty::Bound(_) if !typed && !code.text.starts_with('(') => {
                format!("({}){}", self.ty(&e.ty), code.at(UNARY))
            }
            _ => code.text,
        }
    }

    pub fn dim(&mut self, dim: &DimSpec) -> String {
        match dim {
            DimSpec::Pos(d) => format!("{}_len", ident(&self.program.domains[*d])),
            DimSpec::Bound(d) => format!("{}_len + 1", ident(&self.program.domains[*d])),
            DimSpec::Sym(d) => {
                let dn = ident(&self.program.domains[*d]);
                format!("({dn}_len * ({dn}_len + 1) / 2)")
            }
            DimSpec::Enum(e) => self.program.enums[*e].variants.len().to_string(),
            DimSpec::Int(len) | DimSpec::IntFrom(_, len) => self.expr(len).text,
        }
    }

    /// The arguments of a 1-D vector's constructor: `n, value`.
    pub fn fill_args(&mut self, dims: &[DimSpec], value: Option<&Expr>) -> String {
        self.fill_args_with_layout(dims, value, None)
    }

    pub fn fill_args_with_layout(
        &mut self,
        dims: &[DimSpec],
        value: Option<&Expr>,
        layout: Option<&cpl_ir::NarrowLayout>,
    ) -> String {
        let n = self.dim(&dims[0]);
        match value {
            Some(v) => match layout {
                Some(cpl_ir::NarrowLayout::Seq(inner)) => {
                    let text = self.expr_with_elem_layout(v, Some(inner)).text;
                    format!("{n}, {text}")
                }
                Some(l) => {
                    let text = self.expr(v).text;
                    format!("{n}, {}", self.narrow_cell_value(&text, l))
                }
                None => format!("{n}, {}", self.expr(v).text),
            },
            None => n,
        }
    }

    pub fn expr_with_elem_layout(
        &mut self,
        e: &Expr,
        layout: Option<&cpl_ir::NarrowLayout>,
    ) -> Code {
        let Some(l) = layout else {
            return self.expr(e);
        };
        let elem_ty = match &e.ty {
            Ty::Array { elem, .. } | Ty::List(elem) | Ty::ListView(elem) => Some(elem.as_ref()),
            _ => None,
        };
        let Some(cpp_cell) = elem_ty.and_then(|et| l.cpp_elem(et)) else {
            return self.expr(e);
        };
        match &e.kind {
            ExprKind::Fill(dims, value) => {
                if dims.len() == 1 {
                    let args = self.fill_args_with_layout(dims, value.as_deref(), Some(l));
                    Code::primary(format!("std::vector<{cpp_cell}>({args})"))
                } else {
                    self.need("grid");
                    let lens: Vec<String> = dims.iter().map(|d| self.dim(d)).collect();
                    let val_text = match value {
                        Some(v) => match l {
                            cpl_ir::NarrowLayout::Seq(inner) => {
                                self.expr_with_elem_layout(v, Some(inner)).text
                            }
                            _ => {
                                let t = self.expr(v).text;
                                self.narrow_cell_value(&t, l)
                            }
                        },
                        None => format!("{cpp_cell}()"),
                    };
                    Code::primary(format!(
                        "cpl::grid<{cpp_cell}, {}>({{{}}}, {val_text})",
                        dims.len(),
                        lens.join(", ")
                    ))
                }
            }
            ExprKind::Builtin(Builtin::ListFilled, args) if args.len() == 2 => {
                let n = self.expr(&args[0]).text;
                let v = match l {
                    cpl_ir::NarrowLayout::Seq(inner) => {
                        self.expr_with_elem_layout(&args[1], Some(inner)).text
                    }
                    _ => {
                        let t = self.expr(&args[1]).text;
                        self.narrow_cell_value(&t, l)
                    }
                };
                Code::primary(format!("std::vector<{cpp_cell}>((size_t)({n}), {v})"))
            }
            ExprKind::Tuple(items) if matches!(&e.ty, Ty::List(_)) => {
                if items.is_empty() {
                    Code::primary(format!("std::vector<{cpp_cell}>()"))
                } else {
                    let rendered: Vec<String> = items
                        .iter()
                        .map(|item| match l {
                            cpl_ir::NarrowLayout::Seq(inner) => {
                                self.expr_with_elem_layout(item, Some(inner)).text
                            }
                            _ => {
                                let t = self.expr(item).text;
                                self.narrow_cell_value(&t, l)
                            }
                        })
                        .collect();
                    Code::primary(format!("std::vector<{cpp_cell}>{{{}}}", rendered.join(", ")))
                }
            }
            ExprKind::Read(spec) => {
                if let cpl_ir::Spec::Array { dims, elem } = &**spec {
                    let ty = self.ty_with_elem_layout(&e.ty, Some(l));
                    return self.lambda(&ty, |this, out| {
                        let name = this.temp();
                        this.read_array_with_layout(&name, true, dims, elem, Some(l), out);
                        out.push_str(&format!("{}return {name};\n", pad(this.indent)));
                    });
                }
                self.expr(e)
            }
            ExprKind::Do(block) => {
                let ret = self.ty_with_elem_layout(&e.ty, Some(l));
                self.lambda(&ret, |this, out| this.block(block, out))
            }
            _ => self.expr(e),
        }
    }

    /// Emits `e` with a target `std::array<Cell, K>` representation when `fixed` is `Some(k)`.
    pub fn expr_with_fixed_len(&mut self, e: &Expr, fixed: Option<u32>) -> Code {
        let Some(k) = fixed else {
            return self.expr(e);
        };
        match &e.kind {
            ExprKind::Fill(dims, value) if dims.len() == 1 => {
                let arr_ty = self.fixed_array_cpp(&e.ty, k);
                match value {
                    Some(v) => {
                        let Ty::Array { elem, .. } = &e.ty else {
                            return self.expr(e);
                        };
                        let cell = elem_ty(self.program, self.niches, elem);
                        self.need("filled_array");
                        let val = self.expr(v).text;
                        Code::primary(format!("cpl::filled_array<{cell}, {k}>({val})"))
                    }
                    None => Code::primary(format!("{arr_ty}{{}}")),
                }
            }
            ExprKind::Do(block) => {
                let ret = self.fixed_array_cpp(&e.ty, k);
                self.lambda(&ret, |this, out| this.block(block, out))
            }
            _ => self.expr(e),
        }
    }

    /// What a runtime helper for any element type gave, as the type the
    /// program holds it in: the helper's `std::optional` is packed when
    /// `ty` is.
    fn packed_result(&mut self, ty: &Ty, code: Code) -> Code {
        let cpp = self.ty(ty);
        match ty {
            Ty::Opt(_) if cpp.starts_with("cpl::Opt") => {
                Code::primary(format!("{cpp}({})", code.text))
            }
            _ => code,
        }
    }

    pub fn expr(&mut self, e: &Expr) -> Code {
        let program = self.program;
        if let Some(row) = self.cached_row(crate::rowcache::of_expr(e)) {
            return Code::primary(row);
        }
        match &e.kind {
            // `debug(...)` is for local runs: the judge's C++ has none of it.
            ExprKind::Unit | ExprKind::Error | ExprKind::Debug(..) => Code::primary("(void)0"),
            ExprKind::Int(v) => int_literal(*v),
            ExprKind::Float(v) => float_literal(*v),
            ExprKind::Bool(v) => Code::primary(if *v { "true" } else { "false" }),
            ExprKind::Char(c) => char_literal(*c),
            ExprKind::Str(bytes) => {
                if bytes.contains(&0) {
                    Code::primary(format!(
                        "std::string({}, {})",
                        string_literal(bytes),
                        bytes.len()
                    ))
                } else {
                    Code::primary(format!("std::string({})", string_literal(bytes)))
                }
            }
            ExprKind::None => Code::primary(format!("{}()", self.ty(&e.ty))),
            ExprKind::Some(inner) => {
                let inner = self.expr(inner).text;
                Code::primary(format!("{}({inner})", self.ty(&e.ty)))
            }
            ExprKind::Local(_) if e.ty == Ty::Unit => Code::primary("void()"),
            ExprKind::Local(slot) => Code::primary(self.names[*slot].clone()),
            ExprKind::Global(id) => Code::primary(ident(&program.globals[*id].name)),
            ExprKind::Unary(op, inner) => {
                let code = self.expr(inner);
                let sym = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                    UnOp::Pos => "+",
                    UnOp::BitNot => "~",
                };
                let operand = if code.text.starts_with('-') {
                    format!("({})", code.text)
                } else {
                    code.at(UNARY)
                };
                Code {
                    text: format!("{sym}{operand}"),
                    prec: UNARY,
                    narrow: code.narrow && *op != UnOp::Not,
                }
            }
            ExprKind::Binary(op, l, r) => self.binary(e, *op, l, r),
            // `f == none`, `f != none` on an optional fn: whether one is
            // there. Fns don't compare, so the optionals can't.
            ExprKind::Cmp(op @ (CmpOp::Eq | CmpOp::Ne), x, none)
            | ExprKind::Cmp(op @ (CmpOp::Eq | CmpOp::Ne), none, x)
                if none.kind == ExprKind::None
                    && matches!(&x.ty, Ty::Opt(inner) if matches!(**inner, Ty::Fn(..))) =>
            {
                let code = self.expr(x);
                let has = format!("{}.has_value()", code.at(PRIMARY));
                if *op == CmpOp::Ne {
                    Code::primary(has)
                } else {
                    Code::new(format!("!{has}"), UNARY)
                }
            }
            ExprKind::Cmp(op, l, r) => self.sequence(&[l, r], &Ty::Bool, |_, codes| {
                if let Ty::Struct(id) = l.ty {
                    let sdef = &program.structs[id];
                    match op {
                        CmpOp::Eq | CmpOp::Ne if sdef.eq_fn.is_some() => {
                            let f = fn_name(&program.fns[sdef.eq_fn.unwrap()].name);
                            let call = format!("{f}({}, {})", codes[0].text, codes[1].text);
                            return if *op == CmpOp::Eq {
                                Code::primary(call)
                            } else {
                                Code::new(format!("!{call}"), UNARY)
                            };
                        }
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge if sdef.lt_fn.is_some() => {
                            let f = fn_name(&program.fns[sdef.lt_fn.unwrap()].name);
                            let (a, b) = (&codes[0].text, &codes[1].text);
                            return match op {
                                CmpOp::Lt => Code::primary(format!("{f}({a}, {b})")),
                                CmpOp::Gt => Code::primary(format!("{f}({b}, {a})")),
                                CmpOp::Le => Code::new(format!("!{f}({b}, {a})"), UNARY),
                                CmpOp::Ge => Code::new(format!("!{f}({a}, {b})"), UNARY),
                                _ => unreachable!(),
                            };
                        }
                        _ => {}
                    }
                }
                let (sym, prec) = match op {
                    CmpOp::Ne => ("!=", EQ),
                    CmpOp::Lt => ("<", REL),
                    CmpOp::Le => ("<=", REL),
                    CmpOp::Gt => (">", REL),
                    CmpOp::Ge => (">=", REL),
                    _ => ("==", EQ),
                };
                // `a < b == c` is right but reads wrong: keep the parentheses.
                Code::new(
                    format!("{} {sym} {}", codes[0].at(REL + 1), codes[1].at(REL + 1)),
                    prec,
                )
            }),
            ExprKind::Cast(Cast::Trust, inner) if neighbour(inner).is_some() => {
                // `v.after`, `v.prev`: a position a small step away
                // stays an `int`.
                let (pos, op, step) = neighbour(inner).expect("checked");
                let pos = self.expr(pos);
                let mut code = Code::new(format!("{} {op} {step}", pos.at(ADD)), ADD);
                code.narrow = true;
                code
            }
            ExprKind::Cast(Cast::Raw, inner)
                if matches!(inner.kind, ExprKind::Cast(Cast::FromOrd(_) | Cast::FromLabel(_), _)) =>
            {
                let ExprKind::Cast(cast, ord) = &inner.kind else { unreachable!() };
                let code = self.expr(ord);
                match cast {
                    Cast::FromLabel(d) => Code::new(
                        format!("(int)({} - {}_lo)", code.at(ADD), ident(&program.domains[*d])),
                        UNARY,
                    ),
                    _ => Code::new(format!("(int){}", code.at(UNARY)), UNARY),
                }
            }
            ExprKind::Cast(cast, inner) => {
                let code = self.expr(inner);
                let c_cast = |ty: &str| Code::new(format!("({ty}){}", code.at(UNARY)), UNARY);
                match cast {
                    Cast::IntToFloat | Cast::I128ToFloat | Cast::U64ToFloat => c_cast("double"),
                    Cast::IntToI128 | Cast::U64ToI128 => c_cast("__int128"),
                    Cast::IntToU64 => c_cast("unsigned long long"),
                    Cast::CharToInt | Cast::Ord | Cast::EnumOrd(_) => c_cast("ll"),
                    Cast::OrdToEnum(eid) => {
                        let ename = ident(&self.program.enums[*eid].name);
                        Code::primary(format!("static_cast<{ename}>({})", code.text))
                    }
                    Cast::EnumVal(eid) => {
                        let def = &self.program.enums[*eid];
                        if def.values.iter().any(|v| v.is_some()) {
                            let vals: Vec<String> = def
                                .values
                                .iter()
                                .enumerate()
                                .map(|(i, v)| v.unwrap_or(i as i64).to_string())
                                .collect();
                            Code::primary(format!(
                                "(std::array<ll, {}>{{{}}}[static_cast<size_t>({})])",
                                vals.len(),
                                vals.join(", "),
                                code.text
                            ))
                        } else {
                            c_cast("ll")
                        }
                    }
                    Cast::I128ToU64 => {
                        self.need("narrow");
                        Code::primary(format!("cpl::to_u64({})", code.text))
                    }
                    Cast::I128ToInt | Cast::U64ToInt | Cast::FloatToInt => {
                        self.need("narrow");
                        Code::primary(format!("cpl::to_int({})", code.text))
                    }
                    Cast::IntToChar if matches!(inner.kind, ExprKind::Int(0..=255)) => {
                        c_cast("cpl::u8")
                    }
                    Cast::IntToChar => {
                        self.need("narrow");
                        Code::primary(format!("cpl::to_char({})", code.text))
                    }
                    Cast::StepToChar => {
                        self.need("char_step");
                        Code::primary(format!("cpl::char_step({})", code.text))
                    }
                    Cast::Label(d) => Code::new(
                        format!("{} + {}_lo", code.at(ADD), ident(&program.domains[*d])),
                        ADD,
                    ),
                    Cast::FromLabel(d) => {
                        self.need("at");
                        let d = ident(&program.domains[*d]);
                        Code::primary(format!("cpl::pos({} - {d}_lo, {d}_len)", code.at(ADD)))
                    }
                    Cast::FromOrd(d) => {
                        self.need("at");
                        let d = ident(&program.domains[*d]);
                        Code::primary(format!("cpl::pos({}, {d}_len)", code.text))
                    }
                    Cast::Raw => code,
                    Cast::Trust if matches!(inner.ty, Ty::Pos(_) | Ty::Bound(_)) => code,
                    Cast::Trust => c_cast("int"),
                }
            }
            ExprKind::Index(base, indices)
                if indices.len() == 1
                    && self
                        .found
                        .iter()
                        .any(|(m, k, _)| m.kind == base.kind && k.kind == indices[0].kind) =>
            {
                let it = self
                    .found
                    .iter()
                    .rev()
                    .find(|(m, k, _)| m.kind == base.kind && k.kind == indices[0].kind);
                Code::primary(format!(
                    "{}->second",
                    it.map(|f| f.2.as_str()).unwrap_or_default()
                ))
            }
            ExprKind::Index(base, indices)
                if indices.len() == 1
                    && matches!(
                        &base.kind,
                        ExprKind::Builtin(Builtin::ListConcat, args) if args.len() == 2
                    ) =>
            {
                let ExprKind::Builtin(Builtin::ListConcat, args) = &base.kind else {
                    unreachable!()
                };
                self.need("list_concat_at");
                let operands = [&args[0], &args[1], &indices[0]];
                let is_bool = matches!(&base.ty, Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) if **elem == Ty::Bool);
                self.sequence(&operands, &e.ty, |_, codes| {
                    let call = format!(
                        "cpl::list_concat_at({}, {}, {})",
                        codes[0].text, codes[1].text, codes[2].text
                    );
                    if is_bool {
                        Code::new(format!("(bool){call}"), UNARY)
                    } else {
                        Code::primary(call)
                    }
                })
            }
            ExprKind::Index(base, indices) => {
                let mut operands: Vec<&Expr> = vec![base];
                operands.extend(indices.iter());
                let base_ty = &base.ty;
                let raw_list = matches!(base.kind, ExprKind::Global(id) if self.program.raw_lists.contains(&id))
                    || matches!(indices[0].kind, ExprKind::Cast(Cast::Raw, _))
                    || matches!((&base.kind, &indices[0].kind), (ExprKind::Local(list), ExprKind::Local(i)) if self.inside.contains(&(*list, *i)));
                let checked = !proven(indices) && !raw_list;
                if (matches!(base_ty, Ty::List(_) | Ty::ListView(_) | Ty::MutView(_)) && !raw_list)
                    || (checked && matches!(base_ty, Ty::Array { .. } | Ty::Str | Ty::StrView))
                {
                    if self.expr_fixed_len(base).is_some() {
                        self.need("at_array");
                    } else {
                        self.need("at");
                    }
                }
                if checked && matches!(base_ty, Ty::Range(_)) {
                    self.need("range_at");
                }
                let in_place = self.is_row(base);
                let narrow_elem = self.expr_elem_layout(base).cloned();
                let wide_elem_ty = matches!(narrow_elem, Some(cpl_ir::NarrowLayout::Tuple(_)))
                    .then(|| self.ty(&e.ty));
                self.sequence(&operands, &e.ty, |_, mut codes| {
                    if in_place {
                        if let Some(row) = codes[0].text.strip_suffix(".vec()") {
                            codes[0].text = row.to_string();
                        }
                    }
                    let b = codes[0].at(PRIMARY);
                    let at: Vec<String> = codes[1..]
                        .iter()
                        .map(|c| {
                            if c.text.starts_with('[') {
                                format!("({})", c.text)
                            } else {
                                c.text.clone()
                            }
                        })
                        .collect();
                    let code = match base_ty {
                        Ty::Array { dims, elem } if dims.len() > 1 => {
                            let call = if checked { ".ck" } else { "" };
                            let cell = format!("{b}{call}({})", at.join(", "));
                            if **elem == Ty::Bool {
                                Code::new(format!("(bool){cell}"), UNARY)
                            } else {
                                Code::primary(cell)
                            }
                        }
                        Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) if raw_list && **elem == Ty::Bool => {
                            Code::new(format!("(bool){b}[{}]", at[0]), UNARY)
                        }
                        Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) if raw_list => {
                            Code::primary(format!("{b}[{}]", at[0]))
                        }
                        Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) if **elem == Ty::Bool => Code::new(
                            format!("(bool)cpl::at({}, {})", codes[0].text, at[0]),
                            UNARY,
                        ),
                        Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) => {
                            Code::primary(format!("cpl::at({}, {})", codes[0].text, at[0]))
                        }
                        Ty::Array { elem, .. } if checked && **elem == Ty::Bool => Code::new(
                            format!("(bool)cpl::at({}, {})", codes[0].text, at[0]),
                            UNARY,
                        ),
                        Ty::Array { .. } if checked => {
                            Code::primary(format!("cpl::at({}, {})", codes[0].text, at[0]))
                        }
                        Ty::Str | Ty::StrView if checked => Code::new(
                            format!("(cpl::u8)cpl::at({}, {})", codes[0].text, at[0]),
                            UNARY,
                        ),
                        Ty::Array { elem, .. } if **elem == Ty::Bool => {
                            Code::new(format!("(bool){b}[{}]", at[0]), UNARY)
                        }
                        Ty::Str | Ty::StrView => Code::new(format!("(cpl::u8){b}[{}]", at[0]), UNARY),
                        Ty::Range(_) if checked => {
                            Code::primary(format!("cpl::range_at({}, {})", codes[0].text, at[0]))
                        }
                        Ty::Range(_) => Code::primary(format!("{b}[{}]", at[0])),
                        Ty::Map(..) | Ty::HashMap(..) => {
                            Code::primary(format!("{b}.at({})", at[0]))
                        }
                        _ => Code::primary(format!("{b}[{}]", at[0])),
                    };
                    match &narrow_elem {
                        Some(cpl_ir::NarrowLayout::Int(_)) => {
                            Code::primary(format!("ll({})", code.text))
                        }
                        Some(cpl_ir::NarrowLayout::OptI32) => {
                            Code::primary(format!("cpl::OptInt({})", code.text))
                        }
                        Some(cpl_ir::NarrowLayout::Tuple(_)) => {
                            let wide = wide_elem_ty.as_deref().unwrap_or("auto");
                            Code::primary(format!("{wide}({})", code.text))
                        }
                        _ => code,
                    }
                })
            }
            ExprKind::Field(base, index) => {
                let b = self.expr(base);
                match &base.ty {
                    // A row kept inside the struct is a list to whatever
                    // takes it whole; an index or a length reads it in place.
                    Ty::Struct(id) => {
                        let access = format!(
                            "{}.{}",
                            b.at(PRIMARY),
                            ident(&program.structs[*id].fields[*index].0),
                        );
                        if self.rows.contains_key(&(*id, *index)) {
                            Code::primary(format!("{access}.vec()"))
                        } else if self.narrow_ints.contains_key(&(*id, *index)) {
                            Code::primary(format!("ll({access})"))
                        } else {
                            Code::primary(access)
                        }
                    }
                    _ => Code::primary(format!("std::get<{index}>({})", b.text)),
                }
            }
            ExprKind::Tuple(items) => {
                let ty = self.ty(&e.ty);
                match &e.ty {
                    Ty::Unit | Ty::Error => Code::primary("(void)0"),
                    Ty::Map(..) | Ty::HashMap(..) => Code::primary(format!("{ty}()")),
                    _ if items.is_empty() => Code::primary(format!("{ty}()")),
                    _ => {
                        // `{...}` runs its elements left to right.
                        let struct_id = match e.ty {
                            Ty::Struct(s) => Some(s),
                            _ => None,
                        };
                        let dropped: Vec<bool> = (0..items.len())
                            .map(|idx| struct_id.is_some_and(|s| self.devirt.unread_field(s, idx)))
                            .collect();
                        let items: Vec<String> = items
                            .iter()
                            .enumerate()
                            .filter(|(idx, _)| !dropped[*idx])
                            .map(|(idx, item)| {
                                if struct_id.is_some_and(|s| {
                                    self.devirt.field_elidable(s, idx)
                                        && self.devirt.skippable_value(item)
                                }) {
                                    return "false".to_string();
                                }
                                if let Some(sid) = struct_id {
                                    if let Some(&k) = self.rows.get(&(sid, idx)) {
                                        let cell = self
                                            .ty(&item.ty)
                                            .strip_prefix("std::vector<")
                                            .and_then(|c| c.strip_suffix('>'))
                                            .map(str::to_string);
                                        if let Some(cell) = cell {
                                            if let Some(row_init) =
                                                self.emit_row_init(item, sid, idx, &cell, k)
                                            {
                                                return row_init;
                                            }
                                        }
                                    }
                                }
                                if let Some(l) = struct_id
                                    .and_then(|s| self.elem_layout(cpl_ir::ElemLoc::Field(s, idx)))
                                    .cloned()
                                {
                                    return self.expr_with_elem_layout(item, Some(&l)).text;
                                }
                                let v = self.braced(item);
                                match struct_id.and_then(|s| self.narrow_ints.get(&(s, idx))) {
                                    Some(w) => format!("{}({v})", w.cpp()),
                                    None => v,
                                }
                            })
                            .collect();
                        Code::primary(format!("{ty}{{{}}}", items.join(", ")))
                    }
                }
            }
            ExprKind::Variant(tag, items) => {
                let Ty::Enum(id) = &e.ty else {
                    return Code::primary("(void)0");
                };
                let def = &program.enums[*id];
                let name = ident(&def.name);
                let variant = ident(&def.variants[*tag].0);
                if is_plain_enum(program, *id) {
                    return Code::primary(format!("{name}::{variant}"));
                }
                let operands: Vec<&Expr> = items.iter().collect();
                self.sequence(&operands, &e.ty, |_, codes| {
                    let args: Vec<String> = codes.iter().map(|c| c.text.clone()).collect();
                    Code::primary(format!("{name}::make_{variant}({})", args.join(", ")))
                })
            }
            ExprKind::IsVariant(value, tag) => {
                let Ty::Enum(id) = &value.ty else {
                    return Code::primary("false");
                };
                let def = &program.enums[*id];
                let code = self.expr(value);
                let variant = format!("{}::{}", ident(&def.name), ident(&def.variants[*tag].0));
                if is_plain_enum(program, *id) {
                    Code::new(format!("{} == {variant}", code.at(REL)), EQ)
                } else {
                    Code::new(format!("{}.tag == {variant}", code.at(PRIMARY)), EQ)
                }
            }
            ExprKind::Payload(value, tag, index) => {
                let Ty::Enum(id) = &value.ty else {
                    return Code::primary("(void)0");
                };
                let variant = ident(&program.enums[*id].variants[*tag].0);
                let code = self.expr(value);
                Code::primary(format!("{}.{variant}_{index}", code.at(PRIMARY)))
            }
            ExprKind::IsSome(value) => {
                let code = self.expr(value);
                Code::primary(format!("{}.has_value()", code.at(PRIMARY)))
            }
            // The user's `!` is an assertion: on `none` it stops the
            // program, as it does in `cpl run`. After a test it is a plain read.
            ExprKind::Unwrap(value, true) => {
                self.need("have");
                let code = self.expr(value);
                Code::primary(format!("cpl::have({})", code.text))
            }
            ExprKind::Unwrap(value, false) => {
                let code = self.expr(value);
                Code::new(format!("*{}", code.at(UNARY)), UNARY)
            }
            ExprKind::RangeCheck { value, lo, hi, text } => {
                self.need("check_range");
                let v = self.expr(value);
                let lo_code = match lo {
                    Some(lo) => self.expr(lo).text,
                    None => "-9223372036854775807LL".to_string(),
                };
                let hi_code = match hi {
                    Some(hi) => self.expr(hi).text,
                    None => "9223372036854775807LL".to_string(),
                };
                let text_lit = string_literal(text.as_bytes());
                Code::primary(format!(
                    "cpl::check_range({}, {}, {}, {})",
                    v.text, lo_code, hi_code, text_lit
                ))
            }
            ExprKind::If(cond, then, otherwise) => {
                let c = self.expr(cond);
                let a = self.expr(then);
                let b = self.expr(otherwise);
                if e.ty == Ty::Unit {
                    return Code::primary(format!(
                        "({} ? (void)({}) : (void)({}))",
                        c.at(COND + 1),
                        a.text,
                        b.text
                    ));
                }
                Code {
                    text: format!("{} ? {} : {}", c.at(COND + 1), a.at(COND + 1), b.at(COND)),
                    prec: COND,
                    narrow: a.narrow && b.narrow,
                }
            }
            ExprKind::Do(block) => {
                if let Some(([], place, key_val)) = map_remove_do(e) {
                    self.need("map_take");
                    let p = self.place(place);
                    let k = self.expr(key_val).text;
                    return self.packed_result(
                        &e.ty,
                        Code::primary(format!("cpl::map_take({p}, {k})")),
                    );
                }
                let ret = if let Some(k) = self.expr_fixed_len(e) {
                    self.fixed_array_cpp(&e.ty, k)
                } else if gives_one_closure(block) {
                    // `plan.sorted(by: tree.pos)`: the lambda's own type, not a std::function.
                    "auto".to_string()
                } else {
                    self.ty(&e.ty)
                };
                self.lambda(&ret, |this, out| this.block(block, out))
            }
            ExprKind::Call(id, args) => self.call(e, *id, args),
            ExprKind::CallExpr(callee, args) => {
                let values = args.iter().filter_map(|arg| match arg {
                    Arg::Value(value) => Some(value),
                    Arg::Mut(_) => None,
                });
                let home = match self.closure_id {
                    Some(_) => None,
                    None => self.fn_id,
                };
                let known = self.devirt.target(home, callee);
                if let Some(cid) = known.filter(|_| crate::devirt::skippable(callee)) {
                    self.direct.insert(cid);
                    let operands: Vec<&Expr> = values.collect();
                    return self.sequence(&operands, &e.ty, |this, codes| {
                        let ps = this.fn_value_args(args, codes);
                        Code::primary(format!("{}({})", crate::devirt::name(cid), ps.join(", ")))
                    });
                }
                let mut operands: Vec<&Expr> = vec![callee];
                operands.extend(values);
                self.sequence(&operands, &e.ty, |this, mut codes| {
                    let f = codes.remove(0).at(PRIMARY);
                    let ps = this.fn_value_args(args, codes);
                    Code::primary(format!("{f}({})", ps.join(", ")))
                })
            }
            ExprKind::Closure(cid) => self.emit_closure(*cid, false),
            ExprKind::Range { lo, hi } => {
                self.need("range");
                let rty = self.ty(&e.ty);
                let inner_is_pos = matches!(&e.ty, Ty::Range(inner) if matches!(**inner, Ty::Pos(_) | Ty::Bound(_)));
                self.sequence(&[lo, hi], &e.ty, |_, codes| {
                    if inner_is_pos {
                        Code::primary(format!(
                            "{rty}{{(int)({}), (int)({})}}",
                            codes[0].text, codes[1].text
                        ))
                    } else {
                        Code::primary(format!("{rty}::make({}, {})", codes[0].text, codes[1].text))
                    }
                })
            }
            ExprKind::DomainVal(id) => {
                self.need("domain_val");
                let d = ident(&program.domains[*id]);
                Code::primary(format!("cpl::DomainVal{{{d}_lo, {d}_len}}"))
            }
            ExprKind::Builtin(op, args) => {
                let operands: Vec<&Expr> = args.iter().collect();
                self.sequence(&operands, &e.ty, |this, codes| {
                    let code = this.builtin(e, *op, args, codes);
                    match op {
                        Builtin::First
                        | Builtin::Last
                        | Builtin::MapGet
                        | Builtin::FirstGe
                        | Builtin::FirstGt
                        | Builtin::LastLe
                        | Builtin::LastLt
                        | Builtin::SetMin
                        | Builtin::SetMax => this.packed_result(&e.ty, code),
                        _ => code,
                    }
                })
            }
            ExprKind::Mutate(op, place, args) => {
                fn has_index(p: &Place) -> bool {
                    match &p.kind {
                        PlaceKind::Index(..) => true,
                        PlaceKind::Field(base, _) | PlaceKind::Some(base) => has_index(base),
                        PlaceKind::Local(_) | PlaceKind::Global(_) => false,
                    }
                }
                let impure_args = has_index(place)
                    && args.iter().any(|a| {
                        let mut impure = false;
                        crate::walk::deep_expr(a, &mut |sub| {
                            if matches!(
                                sub.kind,
                                ExprKind::Call(..) | ExprKind::CallExpr(..) | ExprKind::Mutate(..)
                            ) {
                                impure = true;
                            }
                        });
                        impure
                    });
                let narrow_elem = self.place_elem_layout(place).cloned();
                let emit_mutate = |this: &mut Self, p: String, arg: String, arg2: String| -> Code {
                    match op {
                        Mutation::Push if place.ty == Ty::Str => {
                            Code::primary(format!("{p}.push_back((char){arg})"))
                        }
                        Mutation::Push => match &narrow_elem {
                            Some(cpl_ir::NarrowLayout::Seq(inner)) => {
                                let v = this.expr_with_elem_layout(&args[0], Some(inner)).text;
                                Code::primary(format!("{p}.push_back({v})"))
                            }
                            Some(l) => Code::primary(format!(
                                "{p}.push_back({})",
                                this.narrow_cell_value(&arg, l)
                            )),
                            None => Code::primary(format!("{p}.push_back({arg})")),
                        },
                        Mutation::Take => Code::primary(format!("std::exchange({p}, {{}})")),
                        Mutation::Pop => {
                            this.need("pop");
                            this.packed_result(&e.ty, Code::primary(format!("cpl::pop({p})")))
                        }
                        Mutation::PopFirst | Mutation::PopLast => {
                            this.need("ordered_near");
                            let f = if *op == Mutation::PopFirst {
                                "pop_first"
                            } else {
                                "pop_last"
                            };
                            this.packed_result(&e.ty, Code::primary(format!("cpl::{f}({p})")))
                        }
                        Mutation::PopVal => {
                            this.need("pop_val");
                            Code::primary(format!("cpl::pop_val({p})"))
                        }
                        Mutation::Insert => Code::primary(format!("{p}.insert({arg}).second")),
                        Mutation::Remove => Code::new(format!("{p}.erase({arg}) != 0"), EQ),
                        Mutation::Clear => Code::primary(format!("{p}.clear()")),
                        Mutation::Reverse if this.place_is_plain(place) => {
                            Code::primary(format!("std::reverse({p}.begin(), {p}.end())"))
                        }
                        Mutation::Reverse => this.lambda("void", |this, out| {
                            let r = this.temp();
                            out.push_str(&format!("{}auto& {r} = {p};\n", pad(this.indent)));
                            out.push_str(&format!(
                                "{}std::reverse({r}.begin(), {r}.end());\n",
                                pad(this.indent)
                            ));
                        }),
                        // Positions of a domain are already inside; anything
                        // else is held to the length in every build.
                        Mutation::Swap
                            if args
                                .iter()
                                .all(|a| matches!(&a.kind, ExprKind::Cast(Cast::Ord, _))) =>
                        {
                            Code::primary(format!("std::swap({p}[{arg}], {p}[{arg2}])"))
                        }
                        Mutation::Swap => {
                            this.need("swap_at");
                            let is_grid =
                                matches!(&place.ty, Ty::Array { dims, .. } if dims.len() > 1);
                            let seq = if is_grid { format!("{p}.d") } else { p };
                            Code::primary(format!("cpl::swap_at({seq}, {arg}, {arg2})"))
                        }
                        Mutation::Resize if place.ty == Ty::Str => {
                            Code::primary(format!("{p}.resize((size_t)({arg}), (char){arg2})"))
                        }
                        Mutation::Resize => match &narrow_elem {
                            Some(cpl_ir::NarrowLayout::Seq(inner)) => {
                                let v = this.expr_with_elem_layout(&args[1], Some(inner)).text;
                                Code::primary(format!("{p}.resize((size_t)({arg}), {v})"))
                            }
                            Some(l) => Code::primary(format!(
                                "{p}.resize((size_t)({arg}), {})",
                                this.narrow_cell_value(&arg2, l)
                            )),
                            None => Code::primary(format!("{p}.resize((size_t)({arg}), {arg2})")),
                        },
                        Mutation::Truncate => {
                            this.need("truncate");
                            Code::primary(format!("cpl::truncate({p}, {arg})"))
                        }
                        Mutation::Reserve => {
                            this.need("reserve");
                            Code::primary(format!("cpl::reserve({p}, {arg})"))
                        }
                        Mutation::Extend => this.lambda("void", |this, out| {
                            let src = this.temp();
                            let r = this.temp();
                            out.push_str(&format!("{}auto {src} = {arg};\n", pad(this.indent)));
                            out.push_str(&format!("{}auto& {r} = {p};\n", pad(this.indent)));
                            out.push_str(&format!(
                                "{}{r}.insert({r}.end(), {src}.begin(), {src}.end());\n",
                                pad(this.indent)
                            ));
                        }),
                        Mutation::Shuffle => {
                            this.need("rng");
                            Code::primary(format!("cpl::shuffle({p})"))
                        }
                        Mutation::Fill if place.ty == Ty::Str => {
                            Code::primary(format!("std::fill({p}.begin(), {p}.end(), (char){arg})"))
                        }
                        Mutation::Fill => {
                            let is_grid =
                                matches!(&place.ty, Ty::Array { dims, .. } if dims.len() > 1);
                            let vec_expr = if is_grid { format!("{p}.d") } else { p };
                            let v = match &narrow_elem {
                                Some(cpl_ir::NarrowLayout::Seq(inner)) => {
                                    this.expr_with_elem_layout(&args[0], Some(inner)).text
                                }
                                Some(l) => this.narrow_cell_value(&arg, l),
                                None => arg,
                            };
                            Code::primary(format!(
                                "std::fill({vec_expr}.begin(), {vec_expr}.end(), {v})"
                            ))
                        }
                    }
                };
                if impure_args && !matches!(op, Mutation::Extend) {
                    let ret = if e.ty == Ty::Unit {
                        "void".to_string()
                    } else {
                        self.ty(&e.ty)
                    };
                    self.lambda(&ret, |this, out| {
                        let mut eval_args = Vec::with_capacity(args.len());
                        for a in args {
                            let v = this.temp();
                            let code = this.expr(a).text;
                            out.push_str(&format!("{}auto {v} = {code};\n", pad(this.indent)));
                            eval_args.push(v);
                        }
                        let p = this.place(place);
                        let a0 = eval_args.first().cloned().unwrap_or_default();
                        let a1 = eval_args.get(1).cloned().unwrap_or_default();
                        let body = emit_mutate(this, p, a0, a1).text;
                        if e.ty == Ty::Unit {
                            out.push_str(&format!("{}{body};\n", pad(this.indent)));
                        } else {
                            out.push_str(&format!("{}return {body};\n", pad(this.indent)));
                        }
                    })
                } else {
                    let p = self.place(place);
                    let arg = args.first().map(|arg| self.expr(arg).text).unwrap_or_default();
                    let arg2 = args
                        .get(1)
                        .map(|arg| self.expr(arg).text)
                        .unwrap_or_default();
                    emit_mutate(self, p, arg, arg2)
                }
            }
            ExprKind::MutSlice(place, lo, hi) => {
                self.need("mut_view");
                let p = self.place(place);
                let lo = self.expr(lo).text;
                let hi = self.expr(hi).text;
                Code::primary(format!("cpl::mut_slice({p}, {lo}, {hi})"))
            }
            ExprKind::Sort(place, sort_by, desc) => self.lambda("void", |this, out| {
                this.sort_stmts(place, sort_by, desc.as_deref(), out)
            }),
            ExprKind::Read(spec) => self.read(spec),
            ExprKind::Print(args, end) => self.lambda("void", |this, out| {
                this.print_stmts(args, end.as_deref(), out)
            }),
            ExprKind::Format(pieces) => self.lambda("std::string", |this, out| {
                // Values that run code are computed before the capture
                // begins, so what they print goes out and not in.
                let mut values = Vec::new();
                for piece in pieces {
                    let value = match piece {
                        FmtPiece::Value(arg) => Some(&arg.value),
                        FmtPiece::Number(value, _) => Some(value),
                        FmtPiece::Text(_) => None,
                    };
                    if let Some(value) = value {
                        let touch = this.touch(value);
                        let code = this.expr(value);
                        let calls = !crate::walk::Effects::of_expr(value).calls.is_empty();
                        if touch.io || !touch.writes.is_empty() || calls {
                            let name = this.temp();
                            out.push_str(&format!(
                                "{}{} {name} = {};\n",
                                pad(this.indent),
                                this.ty(&value.ty),
                                code.text
                            ));
                            values.push(Code::primary(name));
                        } else {
                            values.push(code);
                        }
                    }
                }
                this.need("capture");
                let capture = this.temp();
                out.push_str(&format!("{}cpl::Capture {capture};\n", pad(this.indent)));
                let mut values = values.into_iter();
                let mut parts = Vec::new();
                for piece in pieces {
                    match piece {
                        FmtPiece::Text(bytes) => parts.push(this.put_text(bytes)),
                        FmtPiece::Value(arg) => {
                            let code = values.next().expect("one per value");
                            parts.push(this.show_stmt(
                                &code,
                                &arg.value.ty,
                                &arg.digits.to_string(),
                            ));
                        }
                        FmtPiece::Number(_, fmt) => {
                            let code = values.next().expect("one per value");
                            this.need("put_num");
                            let (zero, width) = (fmt.zero, fmt.width);
                            parts.push(match fmt.kind {
                                NumKind::Dec => {
                                    format!("cpl::put_num({}, 10, {zero}, {width});", code.text)
                                }
                                NumKind::Hex => {
                                    format!("cpl::put_num({}, 16, {zero}, {width});", code.text)
                                }
                                NumKind::Bin => {
                                    format!("cpl::put_num({}, 2, {zero}, {width});", code.text)
                                }
                                NumKind::Fixed(places) => format!(
                                    "cpl::put_fixed({}, {places}, {zero}, {width});",
                                    code.text
                                ),
                            });
                        }
                    }
                }
                if !parts.is_empty() {
                    out.push_str(&format!("{}{}\n", pad(this.indent), parts.join(" ")));
                }
                out.push_str(&format!("{}return {capture}.take();\n", pad(this.indent)));
            }),
            ExprKind::Fill(dims, value) => {
                let Ty::Array { elem, .. } = &e.ty else {
                    return Code::primary("(void)0");
                };
                let cell = elem_ty(program, self.niches, elem);
                if dims.len() == 1 {
                    let args = self.fill_args(dims, value.as_deref());
                    Code::primary(format!("std::vector<{cell}>({args})"))
                } else {
                    self.need("grid");
                    let lens: Vec<String> = dims.iter().map(|d| self.dim(d)).collect();
                    let value = match value {
                        Some(value) => self.expr(value).text,
                        None => format!("{cell}()"),
                    };
                    Code::primary(format!(
                        "cpl::grid<{cell}, {}>({{{}}}, {value})",
                        dims.len(),
                        lens.join(", ")
                    ))
                }
            }
            ExprKind::DomainLen(d) => Code::primary(format!("{}_len", ident(&program.domains[*d]))),
            ExprKind::DomainLo(d) => Code::primary(format!("{}_lo", ident(&program.domains[*d]))),
        }
    }

    /// Emits a `cpl::Row<{cell}, {k}>` field initializer directly on the stack
    /// when `item` is a fixed-size fill, array comprehension, array literal, or
    /// same-field copy, avoiding a temporary heap `std::vector`.
    pub(super) fn emit_row_init(
        &mut self,
        item: &Expr,
        sid: usize,
        idx: usize,
        cell: &str,
        k: u32,
    ) -> Option<String> {
        let program = self.program;
        if let ExprKind::Builtin(Builtin::ListFilled, fargs) = &item.kind {
            if fargs.len() == 2 {
                let val = self.expr(&fargs[1]).text;
                self.need("row");
                return Some(format!("cpl::Row<{cell}, {k}>::filled({val})"));
            }
        }
        if let ExprKind::Field(base, fidx) = &item.kind {
            if base.ty == Ty::Struct(sid) && *fidx == idx {
                let b = self.expr(base);
                return Some(format!(
                    "{}.{}",
                    b.at(PRIMARY),
                    ident(&program.structs[sid].fields[idx].0)
                ));
            }
        }
        if let ExprKind::Fill(dims, value) = &item.kind {
            if dims.len() == 1 {
                let dim = self.dim(&dims[0]);
                if dim == k.to_string() {
                    self.need("row");
                    return Some(match value {
                        Some(v) => {
                            let val = self.expr(v).text;
                            format!("cpl::Row<{cell}, {k}>::filled({val})")
                        }
                        None => format!("cpl::Row<{cell}, {k}>()"),
                    });
                }
            }
        }
        if let ExprKind::Do(block) = &item.kind {
            let (Stmt::Give(give), rest) = block.stmts.split_last()? else {
                return None;
            };
            let ExprKind::Local(acc) = give.kind else {
                return None;
            };
            if walk::has_give(rest) || self.hoisted.contains(&acc) {
                return None;
            }
            let let_pos = rest.iter().position(|s| {
                matches!(
                    s,
                    Stmt::Let(id, Some(Expr { kind: ExprKind::Fill(dims, _), .. }))
                        if *id == acc && dims.len() == 1
                )
            })?;
            // Only replace `acc` with `cpl::Row` if it is never mutated as a whole `std::vector`.
            let mut safe = true;
            for (pos, stmt) in rest.iter().enumerate() {
                if pos == let_pos {
                    continue;
                }
                walk::deep_stmts(
                    &cpl_ir::Block {
                        stmts: vec![stmt.clone()],
                    },
                    &mut |s| {
                        if let Stmt::Assign(p, ..) = s {
                            if matches!(p.kind, PlaceKind::Local(id) if id == acc) {
                                safe = false;
                            }
                        }
                    },
                );
                walk::deep_exprs(
                    &cpl_ir::Block {
                        stmts: vec![stmt.clone()],
                    },
                    &mut |e| match &e.kind {
                        ExprKind::MutSlice(p, _, _)
                        | ExprKind::Mutate(_, p, _)
                        | ExprKind::Sort(p, _, _)
                            if matches!(p.kind, PlaceKind::Local(id) if id == acc) =>
                        {
                            safe = false;
                        }
                        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                            for arg in args {
                                if matches!(arg, Arg::Mut(p) if matches!(p.kind, PlaceKind::Local(id) if id == acc))
                                {
                                    safe = false;
                                }
                            }
                        }
                        _ => {}
                    },
                );
            }
            if !safe {
                return None;
            }
            self.need("row");
            let row_ty = format!("cpl::Row<{cell}, {k}>");
            let code = self.lambda(&row_ty, |this, out| {
                for (pos, stmt) in rest.iter().enumerate() {
                    if pos == let_pos {
                        if let Stmt::Let(
                            id,
                            Some(Expr {
                                kind: ExprKind::Fill(dims, init),
                                ..
                            }),
                        ) = stmt
                        {
                            let dim = this.dim(&dims[0]);
                            if dim != k.to_string() {
                                this.need("fail");
                                out.push_str(&format!(
                                    "{}if ({dim} != {k}) cpl::fail(\"a row of another length than its field has\");\n",
                                    pad(this.indent)
                                ));
                            }
                            let name = this.names[*id].clone();
                            match init {
                                Some(v) => {
                                    let val = this.expr(v).text;
                                    out.push_str(&format!(
                                        "{}{row_ty} {name} = {row_ty}::filled({val});\n",
                                        pad(this.indent)
                                    ));
                                }
                                None => out.push_str(&format!("{}{row_ty} {name};\n", pad(this.indent))),
                            }
                        }
                    } else {
                        this.stmt(stmt, out);
                    }
                }
                let name = this.names[acc].clone();
                out.push_str(&format!("{}return {name};\n", pad(this.indent)));
            });
            return Some(code.text);
        }
        None
    }

    /// How far a shift goes, held to `0..width` in every build: outside
    /// it C++ gives garbage. A literal inside the range needs no test.
    fn shift_amount(&mut self, b: &Code, width: i64) -> Code {
        if b.text.parse::<i64>().is_ok_and(|n| (0..width).contains(&n)) {
            return Code::new(b.text.clone(), b.prec);
        }
        self.need("shamt");
        Code::primary(format!("cpl::shamt({}, {width})", b.text))
    }

    fn binary(&mut self, e: &Expr, op: BinOp, l: &Expr, r: &Expr) -> Code {
        // Two numbers written out whose result is past an `Int`
        // (`Int.MAX + 1`, the end of `..=Int.MAX`): `cpl run` stops there,
        // and so does this, instead of an overflow the compiler may use.
        if let (Ty::Int, ExprKind::Int(a), ExprKind::Int(b)) = (&e.ty, &l.kind, &r.kind) {
            let past = match op {
                BinOp::Add => a.checked_add(*b),
                BinOp::Sub => a.checked_sub(*b),
                BinOp::Mul => a.checked_mul(*b),
                _ => Some(0),
            }
            .is_none_or(|v| v == i64::MIN);
            if past {
                self.need("fail");
                return Code::primary(
                    "(cpl::fail(\"integer overflow: the result does not fit in an `Int` (64 bits)\"), 0LL)",
                );
            }
        }
        match op {
            BinOp::And => {
                let (a, b) = (self.expr(l), self.expr(r));
                infix(&a, "&&", &b, AND)
            }
            BinOp::Or => {
                let (a, b) = (self.expr(l), self.expr(r));
                // `a || b && c` is right but reads wrong: keep the parentheses.
                Code::new(
                    format!(
                        "{} || {}",
                        a.at(if a.prec == OR { OR } else { AND + 1 }),
                        b.at(AND + 1)
                    ),
                    OR,
                )
            }
            BinOp::Coalesce => {
                // An optional default keeps the result optional: `a` itself.
                // A plain default gives what is inside `a`, which is still
                // an optional when `a` is an optional of optionals.
                let keeps_opt = l.ty == e.ty;
                if walk::path_root(l).is_some() {
                    let a = self.expr(l).at(PRIMARY);
                    let b = self.expr(r);
                    let value = if keeps_opt {
                        a.clone()
                    } else {
                        format!("*{a}")
                    };
                    return Code::new(format!("{a}.has_value() ? {value} : {}", b.at(COND)), COND);
                }
                let ret = self.ty(&e.ty);
                self.lambda(&ret, |this, out| {
                    let a = this.expr(l).text;
                    let name = this.temp();
                    out.push_str(&format!(
                        "{}{} {name} = {a};\n",
                        pad(this.indent),
                        this.ty(&l.ty)
                    ));
                    let b = this.expr(r);
                    let value = if keeps_opt {
                        name.clone()
                    } else {
                        format!("*{name}")
                    };
                    out.push_str(&format!(
                        "{}return {name}.has_value() ? {value} : {};\n",
                        pad(this.indent),
                        b.at(COND)
                    ));
                })
            }
            _ => self.sequence(&[l, r], &e.ty, |this, codes| {
                this.arith(op, l, r, &codes[0], &codes[1])
            }),
        }
    }

    /// `l op r` for operands already emitted as `a` and `b`.
    pub fn arith(&mut self, op: BinOp, l: &Expr, r: &Expr, a: &Code, b: &Code) -> Code {
        let both_narrow = a.narrow && b.narrow;
        match (&l.ty, op) {
            (Ty::Int, BinOp::Add | BinOp::Sub | BinOp::Mul) => {
                let (sym, prec) = match op {
                    BinOp::Add => ("+", ADD),
                    BinOp::Sub => ("-", ADD),
                    _ => ("*", MUL),
                };
                let a = if both_narrow { widen(a) } else { a.clone() };
                infix(&a, sym, b, prec)
            }
            (Ty::Int, BinOp::Div | BinOp::Rem) => {
                let div = op == BinOp::Div;
                let dividend = self.facts.sign(self.fn_id, l);
                let divisor = self.facts.sign(self.fn_id, r);
                if dividend != Sign::Unknown && divisor == Sign::Positive {
                    // Nothing is negative: C++'s own operator agrees.
                    let mut code = infix(a, if div { "/" } else { "%" }, b, MUL);
                    code.narrow = both_narrow;
                    return code;
                }
                let helper = match (divisor == Sign::Positive, div) {
                    (true, true) => "divp",
                    (true, false) => "modp",
                    (false, true) => "ediv",
                    (false, false) => "emod",
                };
                self.need(helper);
                Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::Pow) => {
                self.need("ipow");
                Code::primary(format!("cpl::ipow({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) => {
                let sym = match op {
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    _ => "^",
                };
                let mut code = Code::new(format!("{} {sym} {}", a.at(MUL), b.at(MUL)), BIT);
                code.narrow = both_narrow;
                code
            }
            (Ty::Int, BinOp::Shl) => {
                let b = self.shift_amount(b, 64);
                Code::new(format!("{} << {}", widen(a).at(UNARY), b.at(UNARY)), SHIFT)
            }
            (Ty::Int, BinOp::Shr) => {
                let b = self.shift_amount(b, 64);
                let mut code = Code::new(format!("{} >> {}", a.at(UNARY), b.at(UNARY)), SHIFT);
                code.narrow = a.narrow;
                code
            }
            (Ty::I128, BinOp::Div | BinOp::Rem) => {
                let helper = if op == BinOp::Div {
                    "ediv128"
                } else {
                    "emod128"
                };
                self.need(helper);
                Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
            }
            (Ty::U64, BinOp::Div | BinOp::Rem) => {
                let helper = if op == BinOp::Div { "udiv" } else { "umod" };
                self.need("udiv");
                Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
            }
            (Ty::I128, BinOp::Shl) => {
                self.need("shl128");
                Code::primary(format!("cpl::shl128({}, {})", a.text, b.text))
            }
            (Ty::I128 | Ty::U64, BinOp::Pow) => {
                self.need("wpow");
                Code::primary(format!("cpl::wpow({}, {})", a.text, b.text))
            }
            (Ty::I128 | Ty::U64, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) => {
                let sym = match op {
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    _ => "^",
                };
                Code::new(format!("{} {sym} {}", a.at(MUL), b.at(MUL)), BIT)
            }
            (Ty::I128 | Ty::U64, BinOp::Shl | BinOp::Shr) => {
                let sym = if op == BinOp::Shl { "<<" } else { ">>" };
                let b = self.shift_amount(b, if l.ty == Ty::I128 { 128 } else { 64 });
                Code::new(format!("{} {sym} {}", a.at(UNARY), b.at(UNARY)), SHIFT)
            }
            (Ty::Float, BinOp::Pow) => Code::primary(format!("std::pow({}, {})", a.text, b.text)),
            (Ty::Bool, BinOp::BitAnd) => {
                Code::primary(format!("bool({} & {})", a.at(BIT + 1), b.at(BIT + 1)))
            }
            (Ty::Bool, BinOp::BitOr) => {
                Code::primary(format!("bool({} | {})", a.at(BIT + 1), b.at(BIT + 1)))
            }
            (Ty::Bool, BinOp::BitXor) => Code::new(format!("{} != {}", a.at(REL), b.at(REL)), EQ),
            _ => {
                let (sym, prec) = match op {
                    BinOp::Sub => ("-", ADD),
                    BinOp::Mul => ("*", MUL),
                    BinOp::Div => ("/", MUL),
                    BinOp::Rem => ("%", MUL),
                    _ => ("+", ADD),
                };
                infix(a, sym, b, prec)
            }
        }
    }

    /// The arguments of a call through a fn value: `codes` are the ones
    /// passed by value, in order, and a `mut` one is the place itself.
    fn fn_value_args(&mut self, args: &[Arg], codes: Vec<Code>) -> Vec<String> {
        let mut codes = codes.into_iter();
        args.iter()
            .map(|arg| match arg {
                Arg::Value(_) => codes.next().expect("one per value").text,
                Arg::Mut(place) => self.place(place),
            })
            .collect()
    }

    fn call(&mut self, e: &Expr, id: usize, args: &[Arg]) -> Code {
        let last_use = std::mem::take(&mut self.last_use_call);
        if let Some(code) = self.try_fused_bitset_op(e, id, args) {
            return code;
        }
        let program = self.program;
        let f = &program.fns[id];
        let operands: Vec<&Expr> = args
            .iter()
            .enumerate()
            .filter_map(|(slot, arg)| match arg {
                Arg::Value(value) => {
                    if self.devirt.param_elidable(id, slot)
                        && self.devirt.skippable_value(value)
                    {
                        None
                    } else {
                        Some(value)
                    }
                }
                Arg::Mut(_) => None,
            })
            .collect();
        let lent: Vec<(bool, usize)> = args
            .iter()
            .filter_map(|arg| match arg {
                Arg::Mut(place) => Some(walk::root(place)),
                Arg::Value(_) => None,
            })
            .collect();
        let mut borrowed = Vec::new();
        for (slot, arg) in args.iter().enumerate() {
            if let Arg::Value(value) = arg {
                if let ExprKind::Closure(cid) = value.kind {
                    if fn_param_only_called(program, id, slot) && self.borrow_closures.insert(cid) {
                        borrowed.push(cid);
                    }
                }
            }
        }
        let result = self.sequence(&operands, &e.ty, |this, codes| {
            let mut codes = codes.into_iter();
            let mut texts = Vec::with_capacity(args.len());
            // Arguments for parameters that hold no value: they are run
            // for what they do, ahead of the call, and not passed.
            let mut ran = Vec::new();
            for (slot, arg) in args.iter().enumerate() {
                match arg {
                    Arg::Mut(place) => texts.push(this.place(place)),
                    Arg::Value(value) => {
                        if this.devirt.param_elidable(id, slot)
                            && this.devirt.skippable_value(value)
                        {
                            texts.push("false".to_string());
                            continue;
                        }
                        let code = codes.next().expect("one per value");
                        if slot < f.params.len() && f.locals[slot].ty == Ty::Unit {
                            if code.text != "(void)0" {
                                ran.push(code.text);
                            }
                            continue;
                        }
                        // A parameter taken by reference must not see the
                        // callee change what it refers to: the language
                        // passes values.
                        let by_ref = !value.ty.is_scalar()
                            && this.devirt.template_param_index(id, slot).is_none()
                            && this.facts.stores((Some(id), false, slot)) <= 2
                            && !this.sunk[id].contains(&slot);
                        if last_use && !by_ref && this.moves_in(id, slot, value, args, &code.text) {
                            texts.push(format!("std::move({})", code.text));
                            continue;
                        }
                        let shared = match walk::path_root(value) {
                            Some(root) => {
                                lent.contains(&root)
                                    || (root.0 && this.summaries[id].writes.contains(&root.1))
                            }
                            None => false,
                        };
                        if by_ref && shared {
                            let ty = if let Some(k) = this.expr_fixed_len(value) {
                                this.fixed_array_cpp(&value.ty, k)
                            } else {
                                this.ty_with_elem_layout(&value.ty, this.expr_elem_layout(value))
                            };
                            texts.push(format!("{ty}({})", code.text));
                        } else {
                            texts.push(code.text);
                        }
                    }
                }
            }
            // An argument of a class type makes C++ look in that class's
            // namespace too, where `std::apply` or `std::swap` would join
            // a fn of that name. The qualified name looks only here.
            // No name of the standard library has a capital letter, so a
            // fn with one (`conv_Int_115`, a copy made for its types) can't
            // meet one.
            let name = fn_name(&f.name);
            let scalars = args
                .iter()
                .all(|arg| matches!(arg, Arg::Value(value) if value.ty.is_scalar()));
            let home = if scalars || name.bytes().any(|b| b.is_ascii_uppercase()) {
                ""
            } else {
                "::prog::"
            };
            let call = format!("{home}{name}({})", texts.join(", "));
            if ran.is_empty() {
                Code::primary(call)
            } else {
                Code::primary(format!("({}, {call})", ran.join(", ")))
            }
        });
        for cid in borrowed {
            self.borrow_closures.remove(&cid);
        }
        result
    }

    /// True when `value`, an argument of the call a `return` makes, is a
    /// local the caller owns and names nowhere else in the call, passed to a
    /// parameter taken by value: moving it in skips a copy.
    fn moves_in(&self, id: usize, slot: usize, value: &Expr, args: &[Arg], text: &str) -> bool {
        let ExprKind::Local(local) = value.kind else { return false };
        let Some(caller) = self.fn_id else { return false };
        let cf = &self.program.fns[caller];
        let owned = local >= cf.params.len()
            || (!cf.params[local].mutable
                && (self.sunk[caller].contains(&local) || self.facts.stores((Some(caller), false, local)) > 2));
        if !owned
            || value.ty.is_scalar()
            || matches!(value.ty, Ty::Fn(..))
            || self.devirt.param_known(id, slot)
            || text != self.names[local]
        {
            return false;
        }
        !args.iter().enumerate().any(|(j, arg)| {
            j != slot
                && match arg {
                    Arg::Mut(place) => walk::root(place) == (false, local),
                    Arg::Value(other) => {
                        let mut used = false;
                        walk::deep_expr(other, &mut |e| {
                            used |= matches!(e.kind, ExprKind::Local(s) if s == local);
                        });
                        used
                    }
                }
        })
    }

    /// True for a field read that is a row kept inside its struct.
    fn is_row(&self, e: &Expr) -> bool {
        matches!(&e.kind, ExprKind::Field(base, index)
            if matches!(base.ty, Ty::Struct(id) if self.rows.contains_key(&(id, *index))))
    }

    fn builtin(&mut self, e: &Expr, op: Builtin, args: &[Expr], codes: Vec<Code>) -> Code {
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
            Builtin::Isqrt => {
                self.need("bits");
                Code::primary(format!("cpl::isqrt({})", text(0)))
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
                Code::primary(format!("{}({})", self.ty_with_elem_layout(&e.ty, l), text(0)))
            }
            Builtin::ListFilled => {
                let lty = self.ty(&e.ty);
                Code::primary(format!("{lty}((size_t)({}), {})", text(0), text(1)))
            }
            Builtin::Join => {
                let elem = match &args[1].ty {
                    Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) | Ty::Array { elem, .. } => (**elem).clone(),
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
                Some(Ty::List(_) | Ty::ListView(_) | Ty::MutView(_)) => {
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
                Code::primary(format!("cpl::str_slice({}, {}, {})", text(0), text(1), text(2)))
            }
            Builtin::ListSlice => {
                self.need("list_view");
                Code::primary(format!("cpl::list_slice({}, {}, {})", text(0), text(1), text(2)))
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
            Builtin::RangeLo => Code::primary(format!("{}.lo", codes[0].at(PRIMARY))),
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

    pub(crate) fn emit_closure(&mut self, cid: usize, skip_captures: bool) -> Code {
        let def = &self.program.closures[cid];
        let rec_slot = self.rec_slot.take();
        // Prepare inner names for the closure's locals.
        let mut inner_taken = self.taken.clone();
        let mut inner_names = Vec::with_capacity(def.locals.len());
        let mut inner_fresh = 0usize;
        for local in &def.locals {
            let base = if local.name.is_empty() {
                let mut n;
                loop {
                    n = format!("c{cid}_{inner_fresh}");
                    inner_fresh += 1;
                    if !inner_taken.contains(&n) && !self.top_names.contains(&n) {
                        break;
                    }
                }
                n
            } else {
                let b = ident(&local.name);
                let mut n = b.clone();
                let mut k = 2;
                while inner_taken.contains(&n) || self.top_names.contains(&n) {
                    n = format!("{b}_{k}");
                    k += 1;
                }
                n
            };
            inner_taken.insert(base.clone());
            inner_names.push(base);
        }

        // Build C++17 capture list from enclosing names/globals.
        let mut cap_parts = Vec::new();
        let mut global_ref_aliases = Vec::new();
        let borrow_immutable = self.borrow_closures.contains(&cid);
        if !skip_captures {
            for cap in &def.captures {
                let inner = &inner_names[cap.inner];
                let non_scalar = !def.locals[cap.inner].ty.is_scalar();
                if !cap.is_global && Some(cap.outer) == rec_slot {
                    continue;
                }
                if cap.is_global {
                    let gname = ident(&self.program.globals[cap.outer].name);
                    if cap.by_ref
                        || (borrow_immutable
                            && non_scalar
                            && self.const_globals.contains(&cap.outer))
                    {
                        let constant = if self.const_globals.contains(&cap.outer) { "const " } else { "" };
                        global_ref_aliases.push(format!("{constant}auto& {inner} = {gname};"));
                    } else {
                        cap_parts.push(format!("{inner} = {gname}"));
                    }
                } else {
                    let outer = &self.names[cap.outer];
                    if cap.by_ref || (borrow_immutable && non_scalar) {
                        cap_parts.push(format!("&{inner} = {outer}"));
                    } else {
                        cap_parts.push(format!("{inner} = {outer}"));
                    }
                }
            }
        }

        // Parameters.
        let mut params = Vec::new();
        for cap in &def.captures {
            if !cap.is_global && Some(cap.outer) == rec_slot {
                params.push(("auto&".to_string(), inner_names[cap.inner].clone()));
            }
        }
        for &slot in &def.params {
            let ty = &def.locals[slot].ty;
            let cpp = self.ty(ty);
            let cpp = if def.muts.contains(&slot) {
                format!("{cpp}&")
            } else if ty.is_scalar() {
                cpp
            } else {
                format!("const {cpp}&")
            };
            params.push((cpp, inner_names[slot].clone()));
        }

        let ret_cpp = self.ty(&def.ret);
        let pre_visible = def.params.len() + def.captures.len();
        let inner_hoisted = crate::scope::hoisted(&def.body, pre_visible);

        // Save enclosing emitter state and emit body.
        let saved_closure = self.closure_id.replace(cid);
        let saved_names = std::mem::replace(&mut self.names, inner_names);
        let saved_hoisted = std::mem::replace(&mut self.hoisted, inner_hoisted);
        let saved_in_setup = std::mem::replace(&mut self.in_setup, false);
        let saved_bodies = std::mem::take(&mut self.bodies);
        let inner_spent = crate::scope::spent(self.program, &def.body);
        let saved_spent = std::mem::replace(&mut self.spent, inner_spent);
        let saved_taken = std::mem::replace(&mut self.taken, inner_taken);
        let saved_fresh = std::mem::replace(&mut self.fresh, inner_fresh);
        let saved_brk = std::mem::take(&mut self.break_targets);
        let saved_cont = std::mem::take(&mut self.continue_targets);

        walk::deep_stmts(&def.body, &mut |stmt| match stmt {
            Stmt::Break(Some(id)) => {
                self.break_targets.insert(*id);
            }
            Stmt::Continue(Some(id)) => {
                self.continue_targets.insert(*id);
            }
            _ => {}
        });

        let mut text = String::new();
        self.indent += 1;
        for alias in global_ref_aliases {
            text.push_str(&format!("{}{alias}\n", pad(self.indent)));
        }
        // Declare any hoisted locals inside the closure.
        for slot in self.hoisted.clone() {
            if def.params.contains(&slot)
                || def.captures.iter().any(|c| c.inner == slot)
                || def.locals[slot].ty == Ty::Unit
            {
                continue;
            }
            let ty = &def.locals[slot].ty;
            text.push_str(&format!(
                "{}{} {}{};\n",
                pad(self.indent),
                self.ty(ty),
                self.names[slot],
                self.zero_init(ty)
            ));
        }
        self.block(&def.body, &mut text);
        self.indent -= 1;
        text.push_str(&pad(self.indent));
        text.push('}');
        // A parameter the body never names is left unnamed, as `_` is
        // in the source: g++ warns about a named one.
        let names = |name: &str| {
            let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
            text.match_indices(name).any(|(at, _)| {
                !text[..at].ends_with(word) && !text[at + name.len()..].starts_with(word)
            })
        };
        let param_parts: Vec<String> = params
            .iter()
            .map(|(cpp, name)| {
                if names(name) {
                    format!("{cpp} {name}")
                } else {
                    cpp.clone()
                }
            })
            .collect();
        let text = format!(
            "[{}]({}) mutable -> {ret_cpp} {{\n{text}",
            cap_parts.join(", "),
            param_parts.join(", ")
        );

        // Restore enclosing emitter state.
        self.closure_id = saved_closure;
        self.names = saved_names;
        self.hoisted = saved_hoisted;
        self.in_setup = saved_in_setup;
        self.bodies = saved_bodies;
        self.spent = saved_spent;
        self.taken = saved_taken;
        self.fresh = saved_fresh;
        self.break_targets = saved_brk;
        self.continue_targets = saved_cont;

        Code::primary(text)
    }

    /// Fuse `(a & b & ...).len` / `.count` / `.is_empty` on `Bitset<D>` into a single
    /// zero-allocation word-wise loop over `U64` words.
    fn try_fused_bitset_op(&mut self, e: &Expr, id: usize, args: &[Arg]) -> Option<Code> {
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

fn is_bitset_ty(ty: &Ty, program: &cpl_ir::Program) -> bool {
    let Ty::Struct(sid) = ty else {
        return false;
    };
    let s = &program.structs[*sid];
    s.name.starts_with("Bitset")
        && s.fields.len() == 1
        && s.fields[0].0 == "w"
        && matches!(&s.fields[0].1, Ty::List(elem) if elem.as_ref() == &Ty::U64)
}

/// True when no index is a bare `Int`: each is a position of the
/// dimension's domain, or an `Int` the writer vouched for in `unchecked:`.
fn proven(indices: &[Expr]) -> bool {
    indices.iter().all(|index| {
        index.ty != Ty::Int || matches!(index.kind, ExprKind::Cast(Cast::Raw | Cast::Trust, _))
    })
}

/// Recognizes the IR `Do` block emitted for `m.remove(k)` on a `Map` or `HashMap`:
/// `[...settled_indexes, let key = k, let held = map_get(m, key), if remove(m, key) {}, give held]`.
pub(crate) fn map_remove_do(e: &Expr) -> Option<(&[Stmt], &Place, &Expr)> {
    let ExprKind::Do(block) = &e.kind else { return None };
    let n = block.stmts.len();
    if n < 4 {
        return None;
    }
    let (prefix, tail) = block.stmts.split_at(n - 4);
    let [
        Stmt::Let(key_id, Some(key_val)),
        Stmt::Let(
            held_id,
            Some(Expr { kind: ExprKind::Builtin(Builtin::MapGet, get_args), .. }),
        ),
        Stmt::If(
            Expr { kind: ExprKind::Mutate(Mutation::Remove, place, rem_args), .. },
            then_b,
            else_b,
        ),
        Stmt::Give(Expr { kind: ExprKind::Local(give_id), .. }),
    ] = tail
    else {
        return None;
    };
    if then_b.stmts.is_empty()
        && else_b.stmts.is_empty()
        && give_id == held_id
        && matches!(get_args.as_slice(), [_, Expr { kind: ExprKind::Local(k), .. }] if k == key_id)
        && matches!(rem_args.as_slice(), [Expr { kind: ExprKind::Local(k), .. }] if k == key_id)
        && matches!(place.ty, Ty::Map(..) | Ty::HashMap(..))
    {
        Some((prefix, place, key_val))
    } else {
        None
    }
}

/// True when `param_slot` of `fn_id` is only ever invoked as the direct callee
/// of `CallExpr` (never stored, returned, passed on, mutated, or captured),
/// so a closure passed to it cannot outlive the call.
fn fn_param_only_called(program: &cpl_ir::Program, fn_id: usize, param_slot: usize) -> bool {
    let f = &program.fns[fn_id];
    if param_slot >= f.params.len() || f.params[param_slot].mutable {
        return false;
    }
    let mut total_uses = 0usize;
    let mut indirect_callees = 0usize;
    let mut escapes = false;
    walk::deep_stmts(&f.body, &mut |stmt| {
        if let Stmt::Assign(place, _, _) = stmt {
            if walk::root(place) == (false, param_slot) {
                escapes = true;
            }
        }
    });
    walk::deep_exprs(&f.body, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == param_slot => total_uses += 1,
        ExprKind::CallExpr(callee, args) => {
            if matches!(callee.kind, ExprKind::Local(s) if s == param_slot) {
                indirect_callees += 1;
            }
            for arg in args {
                if let Arg::Mut(place) = arg {
                    if walk::root(place) == (false, param_slot) {
                        escapes = true;
                    }
                }
            }
        }
        ExprKind::Call(_, args) => {
            for arg in args {
                if let Arg::Mut(place) = arg {
                    if walk::root(place) == (false, param_slot) {
                        escapes = true;
                    }
                }
            }
        }
        ExprKind::Closure(cid)
            if program.closures[*cid]
                .captures
                .iter()
                .any(|c| !c.is_global && c.outer == param_slot) =>
        {
            escapes = true;
        }
        _ => {}
    });
    !escapes && total_uses == indirect_callees
}

fn gives_one_closure(block: &cpl_ir::Block) -> bool {
    let mut gives = Vec::new();
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Give(value) = stmt {
            gives.push(matches!(value.kind, ExprKind::Closure(_)));
        }
    });
    gives == [true]
}
