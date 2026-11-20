//! Expressions as C++.

use crate::emit::{
    pad, Code, Emitter, Touch, ADD, AND, BIT, COND, EQ, MUL, OR, PRIMARY, REL, SHIFT, UNARY,
};
use crate::names::{elem_ty, fn_name, ident, is_plain_enum};
use crate::ranges::Sign;
use crate::walk;
use cpl_ir::{
    Arg, BinOp, Builtin, Cast, CmpOp, DimSpec, Expr, ExprKind, FmtPiece, Mutation, NumKind, Place,
    PlaceKind, Stmt, Ty, UnOp,
};

fn int_literal(v: i64) -> Code {
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
            | ExprKind::Print(_)
            | ExprKind::CallExpr(..)
            | ExprKind::Builtin(Builtin::CountQuery | Builtin::SwapInput, _) => touch.io = true,
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
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _) => {
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
                            out.push_str(&format!(
                                "{}return {b}({});\n",
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
                    Ty::Array { dims, .. } if dims.len() > 1 => format!("{b}({})", at.join(", ")),
                    Ty::Map(..) | Ty::HashMap(..) if !store => format!("{b}.at({})", at.join(", ")),
                    Ty::List(_) if matches!(base.kind, PlaceKind::Global(id) if self.program.raw_lists.contains(&id)) => format!("{b}[{}]", at[0]),
                    Ty::List(_) => {
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
            DimSpec::Int(len) | DimSpec::IntFrom(_, len) => self.expr(len).text,
        }
    }

    /// The arguments of a 1-D vector's constructor: `n, value`.
    /// `packed`: the vector is a `?Int` table held as `cpl::OptInt`.
    pub fn fill_args(&mut self, dims: &[DimSpec], value: Option<&Expr>, packed: bool) -> String {
        let n = self.dim(&dims[0]);
        match value.map(|value| (value, self.expr(value).text)) {
            // `none` is what a new cell is.
            Some((value, _)) if packed && value.kind == ExprKind::None => n,
            Some((value, text)) if packed => format!("{n}, {}", self.pack(value, &text)),
            Some((_, text)) => format!("{n}, {text}"),
            None => n,
        }
    }

    /// What a runtime helper for any element type gave, as the type the
    /// program holds it in: the helper's `std::optional` is packed when
    /// `ty` is.
    fn packed_result(&mut self, ty: &Ty, code: Code) -> Code {
        let cpp = self.ty(ty);
        match ty {
            Ty::Opt(_) if cpp.starts_with("cpl::Opt") => Code::primary(format!("{cpp}({})", code.text)),
            _ => code,
        }
    }

    /// True when `e` reads one cell of a packed `?Int` table.
    fn packed_cell(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Index(base, _) => match base.kind {
                ExprKind::Local(slot) => self.is_packed(false, slot),
                ExprKind::Global(id) => self.is_packed(true, id),
                _ => false,
            },
            _ => false,
        }
    }

    /// The C++ of an optional that is about to be tested or opened: a
    /// cell of a packed table is read in place when its indices run no
    /// code, and as a `std::optional` like any other otherwise.
    fn opened(&mut self, e: &Expr) -> Code {
        self.raw_cell = matches!(&e.kind, ExprKind::Index(_, indices) if indices.iter().all(walk::is_simple));
        self.expr(e)
    }

    pub fn expr(&mut self, e: &Expr) -> Code {
        let pack_init = std::mem::take(&mut self.pack_init);
        let raw_cell = std::mem::take(&mut self.raw_cell);
        let program = self.program;
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
            ExprKind::Local(slot) => Code::primary(self.names[*slot].clone()),
            ExprKind::Global(id) => Code::primary(ident(&program.globals[*id].name)),
            ExprKind::Unary(op, inner) => {
                let code = self.expr(inner);
                let sym = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
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
            ExprKind::Cmp(op @ (CmpOp::Eq | CmpOp::Ne), x, none) | ExprKind::Cmp(op @ (CmpOp::Eq | CmpOp::Ne), none, x)
                if none.kind == ExprKind::None && matches!(&x.ty, Ty::Opt(inner) if matches!(**inner, Ty::Fn(..))) =>
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
            ExprKind::Cast(Cast::Trust | Cast::FromOrd(_), inner) if neighbour(inner).is_some() => {
                // `v.after`, `v.prev`: a position a small step away
                // stays an `int`.
                let (pos, op, step) = neighbour(inner).expect("checked");
                let pos = self.expr(pos);
                let mut code = Code::new(format!("{} {op} {step}", pos.at(ADD)), ADD);
                code.narrow = true;
                code
            }
            ExprKind::Cast(cast, inner) => {
                let code = self.expr(inner);
                let c_cast = |ty: &str| Code::new(format!("({ty}){}", code.at(UNARY)), UNARY);
                match cast {
                    Cast::IntToFloat | Cast::I128ToFloat | Cast::U64ToFloat => c_cast("double"),
                    Cast::IntToI128 | Cast::U64ToI128 => c_cast("__int128"),
                    Cast::IntToU64 | Cast::I128ToU64 => c_cast("unsigned long long"),
                    Cast::I128ToInt | Cast::U64ToInt => c_cast("ll"),
                    Cast::FloatToInt | Cast::CharToInt | Cast::Ord => c_cast("ll"),
                    Cast::IntToChar => c_cast("cpl::u8"),
                    Cast::StepToChar => {
                        self.need("char_step");
                        Code::primary(format!("cpl::char_step({})", code.text))
                    }
                    Cast::Label(d) => Code::new(
                        format!("{} + {}_lo", code.at(ADD), ident(&program.domains[*d])),
                        ADD,
                    ),
                    Cast::FromLabel(d) => Code::new(
                        format!(
                            "(int)({} - {}_lo)",
                            code.at(ADD),
                            ident(&program.domains[*d])
                        ),
                        UNARY,
                    ),
                    Cast::FromOrd(_) => c_cast("int"),
                    Cast::Trust if matches!(inner.ty, Ty::Pos(_) | Ty::Bound(_)) => code,
                    Cast::Trust => c_cast("int"),
                }
            }
            ExprKind::Index(base, indices) => {
                let mut operands: Vec<&Expr> = vec![base];
                operands.extend(indices.iter());
                let base_ty = &base.ty;
                let raw_list = matches!(base.kind, ExprKind::Global(id) if self.program.raw_lists.contains(&id));
                if matches!(base_ty, Ty::List(_)) && !raw_list {
                    self.need("at");
                }
                // A cell of a packed table is a `std::optional` to whatever
                // reads it, unless it is only being tested or opened.
                let unpack = self.packed_cell(e) && !raw_cell;
                self.sequence(&operands, &e.ty, |_, codes| {
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
                    let cell = match base_ty {
                        Ty::Array { dims, elem } if dims.len() > 1 => {
                            let cell = format!("{b}({})", at.join(", "));
                            if **elem == Ty::Bool {
                                Code::new(format!("(bool){cell}"), UNARY)
                            } else {
                                Code::primary(cell)
                            }
                        }
                        Ty::List(elem) if raw_list && **elem == Ty::Bool => {
                            Code::new(format!("(bool){b}[{}]", at[0]), UNARY)
                        }
                        Ty::List(_) if raw_list => Code::primary(format!("{b}[{}]", at[0])),
                        Ty::List(elem) if **elem == Ty::Bool => Code::new(
                            format!("(bool)cpl::at({}, {})", codes[0].text, at[0]),
                            UNARY,
                        ),
                        Ty::List(_) => {
                            Code::primary(format!("cpl::at({}, {})", codes[0].text, at[0]))
                        }
                        Ty::Array { elem, .. } if **elem == Ty::Bool => {
                            Code::new(format!("(bool){b}[{}]", at[0]), UNARY)
                        }
                        Ty::Str => Code::new(format!("(cpl::u8){b}[{}]", at[0]), UNARY),
                        Ty::Map(..) | Ty::HashMap(..) => {
                            Code::primary(format!("{b}.at({})", at[0]))
                        }
                        _ => Code::primary(format!("{b}[{}]", at[0])),
                    };
                    match unpack {
                        true => Code::primary(format!("std::optional<ll>({})", cell.text)),
                        false => cell,
                    }
                })
            }
            ExprKind::Field(base, index) => {
                let b = self.expr(base);
                match &base.ty {
                    Ty::Struct(id) => Code::primary(format!(
                        "{}.{}",
                        b.at(PRIMARY),
                        ident(&program.structs[*id].fields[*index].0)
                    )),
                    _ => Code::primary(format!("std::get<{index}>({})", b.text)),
                }
            }
            ExprKind::Tuple(items) => {
                let ty = self.ty(&e.ty);
                match &e.ty {
                    Ty::Unit | Ty::Error => Code::primary("(void)0"),
                    Ty::Map(..) | Ty::HashMap(..) => Code::primary(format!("{ty}()")),
                    _ if items.is_empty() && pack_init => {
                        Code::primary(format!("{}()", self.packed_ty(&e.ty)))
                    }
                    _ if items.is_empty() => Code::primary(format!("{ty}()")),
                    _ => {
                        // `{...}` runs its elements left to right.
                        let items: Vec<String> =
                            items.iter().map(|item| self.braced(item)).collect();
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
                let code = self.opened(value);
                Code::primary(format!("{}.has_value()", code.at(PRIMARY)))
            }
            // The user's `!` is an assertion: on `none` it stops the
            // program, as it does in `cpl run`. After a test it is a plain read.
            ExprKind::Unwrap(value, true) => {
                self.need("have");
                let code = self.opened(value);
                Code::primary(format!("cpl::have({})", code.text))
            }
            ExprKind::Unwrap(value, false) => {
                let code = self.opened(value);
                Code::new(format!("*{}", code.at(UNARY)), UNARY)
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
                let ret = self.ty(&e.ty);
                self.lambda(&ret, |this, out| this.block(block, out))
            }
            ExprKind::Call(id, args) => self.call(e, *id, args),
            ExprKind::CallExpr(callee, args) => {
                let home = match self.closure_id {
                    Some(_) => None,
                    None => self.fn_id,
                };
                let known = self.devirt.target(home, callee);
                if let Some(cid) = known.filter(|_| crate::devirt::skippable(callee)) {
                    self.direct.insert(cid);
                    let operands: Vec<&Expr> = args.iter().collect();
                    return self.sequence(&operands, &e.ty, |_, codes| {
                        let ps: Vec<String> = codes.iter().map(|c| c.text.clone()).collect();
                        Code::primary(format!("{}({})", crate::devirt::name(cid), ps.join(", ")))
                    });
                }
                let mut operands: Vec<&Expr> = vec![callee];
                operands.extend(args.iter());
                self.sequence(&operands, &e.ty, |_, codes| {
                    let f = codes[0].at(PRIMARY);
                    let ps: Vec<String> = codes[1..].iter().map(|c| c.text.clone()).collect();
                    Code::primary(format!("{f}({})", ps.join(", ")))
                })
            }
            ExprKind::Closure(cid) => self.emit_closure(*cid),
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
                        Code::primary(format!("{rty}{{{}, {}}}", codes[0].text, codes[1].text))
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
                let p = self.place(place);
                let (global, index) = walk::root(place);
                let packed = self.is_packed(global, index);
                let arg = args
                    .first()
                    .map(|arg| {
                        let text = self.expr(arg).text;
                        match packed {
                            true => self.pack(arg, &text),
                            false => text,
                        }
                    })
                    .unwrap_or_default();
                let arg2 = args
                    .get(1)
                    .map(|arg| self.expr(arg).text)
                    .unwrap_or_default();
                match op {
                    Mutation::Push if place.ty == Ty::Str => {
                        Code::primary(format!("{p}.push_back((char){arg})"))
                    }
                    Mutation::Push => Code::primary(format!("{p}.push_back({arg})")),
                    Mutation::Take => Code::primary(format!("std::exchange({p}, {{}})")),
                    Mutation::Pop => {
                        self.need("pop");
                        self.packed_result(&e.ty, Code::primary(format!("cpl::pop({p})")))
                    }
                    Mutation::PopVal => {
                        self.need("pop_val");
                        Code::primary(format!("cpl::pop_val({p})"))
                    }
                    Mutation::Insert => Code::primary(format!("{p}.insert({arg}).second")),
                    Mutation::Remove => Code::new(format!("{p}.erase({arg}) != 0"), EQ),
                    Mutation::Clear => Code::primary(format!("{p}.clear()")),
                    Mutation::Reverse if self.place_is_plain(place) => {
                        Code::primary(format!("std::reverse({p}.begin(), {p}.end())"))
                    }
                    Mutation::Reverse => self.lambda("void", |this, out| {
                        let r = this.temp();
                        out.push_str(&format!("{}auto& {r} = {p};\n", pad(this.indent)));
                        out.push_str(&format!(
                            "{}std::reverse({r}.begin(), {r}.end());\n",
                            pad(this.indent)
                        ));
                    }),
                    Mutation::Swap => Code::primary(format!("std::swap({p}[{arg}], {p}[{arg2}])")),
                    Mutation::Resize => Code::primary(format!("{p}.resize((size_t)({arg}), {arg2})")),
                    Mutation::Truncate => Code::primary(format!(
                        "({p}.size() > (size_t)std::max<ll>(0, {arg}) ? {p}.resize((size_t)std::max<ll>(0, {arg})) : (void)0)"
                    )),
                    Mutation::Extend => self.lambda("void", |this, out| {
                        let r = this.temp();
                        let src = this.temp();
                        out.push_str(&format!("{}auto& {r} = {p};\n", pad(this.indent)));
                        out.push_str(&format!("{}auto {src} = {arg};\n", pad(this.indent)));
                        out.push_str(&format!(
                            "{}{r}.insert({r}.end(), {src}.begin(), {src}.end());\n",
                            pad(this.indent)
                        ));
                    }),
                    Mutation::Shuffle => {
                        self.need("rng");
                        Code::primary(format!("cpl::shuffle({p})"))
                    }
                    Mutation::Fill => {
                        let is_grid = matches!(&place.ty, Ty::Array { dims, .. } if dims.len() > 1);
                        let vec_expr = if is_grid { format!("{p}.d") } else { p };
                        Code::primary(format!("std::fill({vec_expr}.begin(), {vec_expr}.end(), {arg})"))
                    }
                }
            }
            ExprKind::Sort(place, sort_by) => {
                self.lambda("void", |this, out| this.sort_stmts(place, sort_by, out))
            }
            ExprKind::Read(spec) => self.read(spec),
            ExprKind::Print(args) => self.lambda("void", |this, out| this.print_stmts(args, out)),
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
                let cell = match pack_init {
                    true => "cpl::OptInt".to_string(),
                    false => elem_ty(program, self.niches, elem),
                };
                if dims.len() == 1 {
                    let args = self.fill_args(dims, value.as_deref(), pack_init);
                    Code::primary(format!("std::vector<{cell}>({args})"))
                } else {
                    self.need("grid");
                    let lens: Vec<String> = dims.iter().map(|d| self.dim(d)).collect();
                    let value = match value {
                        Some(value) => {
                            let text = self.expr(value).text;
                            match pack_init {
                                true => self.pack(value, &text),
                                false => text,
                            }
                        }
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
                let mut code = Code::new(format!("{} {sym} {}", a.at(BIT + 1), b.at(BIT + 1)), BIT);
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
                let helper = if op == BinOp::Div { "ediv128" } else { "emod128" };
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
                Code::new(format!("{} {sym} {}", a.at(BIT + 1), b.at(BIT + 1)), BIT)
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

    fn call(&mut self, e: &Expr, id: usize, args: &[Arg]) -> Code {
        let program = self.program;
        let f = &program.fns[id];
        let operands: Vec<&Expr> = args
            .iter()
            .filter_map(|arg| match arg {
                Arg::Value(value) => Some(value),
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
        self.sequence(&operands, &e.ty, |this, codes| {
            let mut codes = codes.into_iter();
            let mut texts = Vec::with_capacity(args.len());
            // Arguments for parameters that hold no value: they are run
            // for what they do, ahead of the call, and not passed.
            let mut ran = Vec::new();
            for (slot, arg) in args.iter().enumerate() {
                match arg {
                    Arg::Mut(place) => texts.push(this.place(place)),
                    Arg::Value(value) => {
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
                            && this.facts.stores((Some(id), false, slot)) <= 2;
                        let shared = match walk::path_root(value) {
                            Some(root) => {
                                lent.contains(&root)
                                    || (root.0 && this.summaries[id].writes.contains(&root.1))
                            }
                            None => false,
                        };
                        if by_ref && shared {
                            texts.push(format!("{}({})", this.ty(&value.ty), code.text));
                        } else {
                            texts.push(code.text);
                        }
                    }
                }
            }
            // An argument of a class type makes C++ look in that class's
            // namespace too, where `std::apply` or `std::swap` would join
            // a fn of that name. The qualified name looks only here.
            let home = if args.iter().all(|arg| matches!(arg, Arg::Value(value) if value.ty.is_scalar())) {
                ""
            } else {
                "::prog::"
            };
            let call = format!("{home}{}({})", fn_name(&f.name), texts.join(", "));
            if ran.is_empty() {
                Code::primary(call)
            } else {
                Code::primary(format!("({}, {call})", ran.join(", ")))
            }
        })
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
            Builtin::Abs if e.ty == Ty::Float => Code::primary(format!("std::fabs({})", text(0))),
            Builtin::Abs if e.ty == Ty::I128 => {
                self.need("abs128");
                Code::primary(format!("cpl::abs128({})", text(0)))
            }
            Builtin::Abs => Code::primary(format!("std::abs({})", widen(&codes[0]).text)),
            Builtin::Sqrt => Code::primary(format!("std::sqrt({})", text(0))),
            Builtin::Isqrt => {
                self.need("bits");
                Code::primary(format!("cpl::isqrt({})", text(0)))
            }
            Builtin::Flush => {
                self.need("out");
                Code::primary("(cpl::flush(), fflush(stdout))".to_string())
            }
            Builtin::Atan2 => Code::primary(format!("std::atan2({}, {})", text(0), text(1))),
            Builtin::Floor => Code::new(format!("(ll)std::floor({})", text(0)), UNARY),
            Builtin::Ceil => Code::new(format!("(ll)std::ceil({})", text(0)), UNARY),
            Builtin::Round => Code::new(format!("(ll)std::round({})", text(0)), UNARY),
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
            Builtin::Copy => Code::primary(format!("{}({})", self.ty(&e.ty), text(0))),
            Builtin::ListFilled => {
                let lty = self.ty(&e.ty);
                Code::primary(format!("{lty}((size_t)({}), {})", text(0), text(1)))
            }
            Builtin::Join => {
                let elem = match &args[1].ty {
                    Ty::List(elem) | Ty::Array { elem, .. } => (**elem).clone(),
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
                            let some = this.show_stmt(&Code::new("*item", UNARY), inner, "6");
                            format!("if (item) {{ {some} }} else cpl::put_str({text});")
                        }
                        _ => this.show_stmt(&item, &elem, "6"),
                    };
                    out.push_str(&format!("{lead}    {stmt}\n{lead}}}\n"));
                    out.push_str(&format!("{lead}return {cap}.take();\n"));
                })
            }
            Builtin::SeqGet => {
                let opt = self.ty(&e.ty);
                let (seq, at) = (text(0), text(1));
                Code::primary(format!(
                    "([&]() -> {opt} {{ const auto& s = {seq}; ll i = {at}; return i < 0 || i >= (ll)s.size() ? {opt}() : {opt}(s[i]); }}())"
                ))
            }
            Builtin::ShowStr => {
                let arg = &args[0];
                let code = codes[0].clone();
                self.lambda("std::string", |this, out| {
                    this.need("capture");
                    let cap = this.temp();
                    out.push_str(&format!("{}cpl::Capture {cap};\n", pad(this.indent)));
                    let stmt = this.show_stmt(&code, &arg.ty, "6");
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
            Builtin::Len if is_grid => {
                Code::new(format!("(ll){}.d.size()", codes[0].at(PRIMARY)), UNARY)
            }
            Builtin::Len if matches!(first_ty, Some(Ty::Range(_))) => {
                Code::primary(format!("{}.len()", codes[0].at(PRIMARY)))
            }
            Builtin::Len if matches!(first_ty, Some(Ty::DomainVal)) => {
                Code::primary(format!("{}.len", codes[0].at(PRIMARY)))
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
            Builtin::FirstVal => helper(self, "first_val"),
            Builtin::LastVal => helper(self, "last_val"),
            Builtin::Contains => {
                let holder = codes[0].at(PRIMARY);
                match first_ty {
                    Some(Ty::Range(_)) => Code::primary(format!("{holder}.has({})", text(1))),
                    Some(Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..)) => {
                        Code::new(format!("{holder}.count({}) != 0", text(1)), EQ)
                    }
                    Some(Ty::Str) => Code::new(
                        format!(
                            "{holder}.find((char){}) != std::string::npos",
                            codes[1].at(UNARY)
                        ),
                        EQ,
                    ),
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
            Builtin::SetMin => helper(self, "set_min"),
            Builtin::SetMax => helper(self, "set_max"),
            Builtin::Sorted => helper(self, "sorted"),
            Builtin::Reversed => {
                if matches!(first_ty, Some(Ty::Str)) {
                    let s = text(0);
                    Code::primary(format!(
                        "([](std::string s){{ std::reverse(s.begin(), s.end()); return s; }}({s}))"
                    ))
                } else {
                    helper(self, "reversed")
                }
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
        }
    }

    pub(crate) fn emit_closure(&mut self, cid: usize) -> Code {
        let def = &self.program.closures[cid];
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
        for cap in &def.captures {
            let inner = &inner_names[cap.inner];
            if cap.is_global {
                let gname = ident(&self.program.globals[cap.outer].name);
                if cap.by_ref {
                    global_ref_aliases.push(format!("auto& {inner} = {gname};"));
                } else {
                    cap_parts.push(format!("{inner} = {gname}"));
                }
            } else {
                let outer = &self.names[cap.outer];
                if cap.by_ref {
                    cap_parts.push(format!("&{inner} = {outer}"));
                } else {
                    cap_parts.push(format!("{inner} = {outer}"));
                }
            }
        }

        // Parameters.
        let mut params = Vec::new();
        for &slot in &def.params {
            let ty = &def.locals[slot].ty;
            let cpp = self.ty(ty);
            let cpp = if ty.is_scalar() {
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
        self.spent = saved_spent;
        self.taken = saved_taken;
        self.fresh = saved_fresh;
        self.break_targets = saved_brk;
        self.continue_targets = saved_cont;

        Code::primary(text)
    }
}
