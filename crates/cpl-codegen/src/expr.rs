//! Expressions as C++.

use crate::emit::{
    order_fn, pad, Code, Emitter, Touch, ADD, AND, BIT, COND, EQ, MUL, OR, PRIMARY, REL, SHIFT,
    UNARY,
};
use crate::names::{
    bits_field, cast, dom_ident, elem_ty, fn_name, global_ident, ident, is_plain_enum,
};
use crate::walk;
use std::collections::BTreeMap;
pub(crate) const FLOAT_DIGITS: &str = "10";

use cpl_ir::{
    Arg, BinOp, Builtin, Cast, CmpOp, DimSpec, Expr, ExprKind, FmtPiece, Mutation, NumKind, Place,
    PlaceKind, Stmt, Ty, UnOp,
};

mod closure;

mod builtin;

mod arith;

mod place;

mod layout;

mod rows;

mod call;

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

/// The place a block names and does nothing else with, `{ let t = a._xs[i]; t }` from an
/// inlined `lends` call: read where it is, so `a.at(i).v` copies no node.
fn gives_its_place<'e>(block: &'e cpl_ir::Block, ty: &Ty) -> Option<&'e Expr> {
    let [Stmt::Let(slot, Some(init)), Stmt::Give(given)] = block.stmts.as_slice() else {
        return None;
    };
    (matches!(given.kind, ExprKind::Local(s) if s == *slot)
        && !ty.is_scalar()
        && crate::stmt::place_root(init).is_some()
        && Touch::of_expr(init).quiet())
    .then_some(init)
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

pub(crate) fn infix(l: &Code, sym: &str, r: &Code, prec: u8) -> Code {
    Code::new(format!("{} {sym} {}", l.at(prec), r.at(prec + 1)), prec)
}

/// `e` as a position and a small step from it, when it is one.
/// `(a ++ b)[i]` as its operands `[a, b, i]`.
fn concat_at<'e>(base: &'e Expr, indices: &'e [Expr]) -> Option<[&'e Expr; 3]> {
    match (&base.kind, indices) {
        (ExprKind::Builtin(Builtin::ListConcat, args), [i]) => match args.as_slice() {
            [a, b] => Some([a, b, i]),
            _ => None,
        },
        _ => None,
    }
}

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
        self.sequence_laid(operands, &[], ret, build)
    }

    /// `sequence`, building operand `i` in `layouts[i]`: a fresh list given
    /// for a narrowed parameter is made in the parameter's cells.
    pub fn sequence_laid(
        &mut self,
        operands: &[&Expr],
        layouts: &[Option<cpl_ir::NarrowLayout>],
        ret: &Ty,
        build: impl FnOnce(&mut Self, Vec<Code>) -> Code,
    ) -> Code {
        let laid =
            |this: &mut Self, i: usize, e: &Expr| match layouts.get(i).and_then(|l| l.as_ref()) {
                Some(l) => this.expr_with_elem_layout(e, Some(l)),
                None => this.expr(e),
            };
        let touches: Vec<Touch> = operands.iter().map(|e| self.touch(e)).collect();
        let n = operands.len();
        let ordered = (0..n).any(|i| (i + 1..n).any(|j| touches[i].conflicts(&touches[j])));
        if !ordered {
            let codes = (0..n).map(|i| laid(self, i, operands[i])).collect();
            return build(self, codes);
        }
        let ret_text = self.ty(ret);
        let is_unit = *ret == Ty::Unit;
        self.lambda(&ret_text, |this, out| {
            let mut codes = Vec::with_capacity(n);
            for (i, e) in operands.iter().enumerate() {
                let code = laid(this, i, e);
                if i + 1 == n
                    || this.is_fixed(e)
                    || matches!(e.kind, ExprKind::Str(_) | ExprKind::None)
                {
                    codes.push(code);
                } else if e.ty == Ty::Unit {
                    if code.text != "(void)0" {
                        out.push_str(&format!("{}{};\n", pad(this.indent), code.text));
                    }
                    codes.push(Code::primary("(void)0"));
                } else {
                    this.note_operand_copy(e);
                    let name = this.temp();
                    // A closure keeps its own lambda type: the callee takes it as written.
                    let ty = match e.kind {
                        ExprKind::Closure(_) => "auto".to_string(),
                        _ => this.ty(&e.ty),
                    };
                    out.push_str(&format!(
                        "{}{ty} {name} = {};\n",
                        pad(this.indent),
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

    pub fn expr(&mut self, e: &Expr) -> Code {
        let program = self.program;
        if let Some((_, owner)) = self.pinned.iter().find(|(p, _)| std::ptr::eq(*p, e)) {
            return Code::primary(owner.clone());
        }
        if let Some(row) = self.cached_row(crate::rowcache::of_expr(e)) {
            return Code::primary(row);
        }
        match &e.kind {
            ExprKind::Binary(BinOp::Coalesce, ..)
                if self.moved_entry.remove(&(e as *const _ as usize)) =>
            {
                self.moved_entry_code(e)
            }
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
            ExprKind::Some(inner) if inner.ty == Ty::Unit => {
                let inner = self.expr(inner).text;
                Code::primary(format!("((void)({inner}), true)"))
            }
            ExprKind::Some(inner) => {
                let text = self.expr(inner).text;
                self.note_some_copy(inner, &text);
                Code::primary(format!("{}({text})", self.ty(&e.ty)))
            }
            ExprKind::Local(_) if e.ty == Ty::Unit => Code::primary("void()"),
            ExprKind::Local(slot)
                if self.pushed_moves.contains(&(e as *const Expr))
                    && !self.ref_bound.contains(slot)
                    && crate::lend::holds_heap(self.program, &e.ty) =>
            {
                Code::primary(format!("std::move({})", self.names[*slot]))
            }
            ExprKind::Local(slot) => Code::primary(self.names[*slot].clone()),
            ExprKind::Global(id) => match self.devirt.global_target(*id) {
                Some(cid) => {
                    self.direct.insert(cid);
                    Code::primary(crate::devirt::name(cid))
                }
                None => Code::primary(global_ident(program, *id)),
            },
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
                if none.kind == ExprKind::None && matches!(&x.ty, Ty::Opt(_)) =>
            {
                let has = self.has_value(x);
                if *op == CmpOp::Ne {
                    Code::primary(has)
                } else {
                    Code::new(format!("!{has}"), UNARY)
                }
            }
            ExprKind::Cmp(op, l, r) => self.sequence(&[l, r], &Ty::Bool, |_, codes| {
                // `()` has one value: equal to itself, so never less or greater.
                if l.ty == Ty::Unit {
                    let holds = matches!(op, CmpOp::Eq | CmpOp::Le | CmpOp::Ge);
                    let (a, b) = (&codes[0].text, &codes[1].text);
                    return Code::primary(format!("((void)({a}), (void)({b}), {holds})"));
                }
                if let Ty::Struct(id) = l.ty {
                    let sdef = &program.structs[id];
                    match op {
                        CmpOp::Eq | CmpOp::Ne if let Some(eq_fn) = sdef.eq_fn => {
                            let f = fn_name(&program.fns[eq_fn].name);
                            let call = format!("{f}({}, {})", codes[0].text, codes[1].text);
                            return if *op == CmpOp::Eq {
                                Code::primary(call)
                            } else {
                                Code::new(format!("!{call}"), UNARY)
                            };
                        }
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge => {
                            if let Some(ord) = sdef.ord {
                                let f = order_fn(program, ord);
                                let (a, b) = (&codes[0].text, &codes[1].text);
                                // `a > b` is `b < a`; `a <= b` is `!(b < a)`.
                                let swap = matches!(op, CmpOp::Gt | CmpOp::Le);
                                let (x, y) = if swap { (b, a) } else { (a, b) };
                                let less = format!("{f}({x}, {y})");
                                return match op {
                                    CmpOp::Le | CmpOp::Ge => Code::new(format!("!{less}"), UNARY),
                                    _ => Code::primary(less),
                                };
                            }
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
            ExprKind::Cast(Cast::Trust, inner) if let Some((pos, op, step)) = neighbour(inner) => {
                // `v.after`, `v.prev`: a position a small step away
                // stays an `int`.
                let pos = self.expr(pos);
                let mut code = Code::new(format!("{} {op} {step}", pos.at(ADD)), ADD);
                code.narrow = true;
                code
            }
            ExprKind::Cast(Cast::Raw, inner)
                if let ExprKind::Cast(cast @ (Cast::FromOrd(_) | Cast::FromLabel(_)), ord) =
                    &inner.kind =>
            {
                let code = self.expr(ord);
                match cast {
                    Cast::FromLabel(d) => Code::new(
                        format!(
                            "({})({} - {}_lo)",
                            crate::width::pos_cpp(program, *d),
                            code.at(ADD),
                            dom_ident(program, *d)
                        ),
                        UNARY,
                    ),
                    Cast::FromOrd(d) => Code::new(
                        format!("({}){}", crate::width::pos_cpp(program, *d), code.at(UNARY)),
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
                    Cast::IntToChar if matches!(inner.kind, ExprKind::Int(0..=127)) => {
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
                        format!("{} + {}_lo", code.at(ADD), dom_ident(program, *d)),
                        ADD,
                    ),
                    Cast::FromLabel(d) => {
                        self.need("at");
                        let d = dom_ident(program, *d);
                        let pos = crate::width::pos_fn(program, e);
                        Code::primary(format!("{pos}({} - {d}_lo, {d}_len)", code.at(ADD)))
                    }
                    Cast::FromOrd(d) if self.ord_inside(*d, inner) => Code::new(
                        format!("({}){}", crate::width::pos_cpp(program, *d), code.at(UNARY)),
                        UNARY,
                    ),
                    Cast::FromOrd(d) => {
                        self.need("at");
                        let d = dom_ident(program, *d);
                        let pos = crate::width::pos_fn(program, e);
                        Code::primary(format!("{pos}({}, {d}_len)", code.text))
                    }
                    Cast::Raw if inner.ty == Ty::U64 => c_cast("ll"),
                    Cast::Raw => code,
                    Cast::Trust if matches!(inner.ty, Ty::Pos(_) | Ty::Bound(_)) => code,
                    Cast::Trust => c_cast(match e.ty {
                        Ty::Pos(d) | Ty::Bound(d) => crate::width::pos_cpp(program, d),
                        _ => "int",
                    }),
                }
            }
            ExprKind::Index(base, indices)
                if indices.len() == 1
                    && self.found.iter().any(|(m, k, _)| {
                        crate::stmt::same_path(m, base) && crate::stmt::same_path(k, &indices[0])
                    }) =>
            {
                let cell = self.found.iter().rev().find(|(m, k, _)| {
                    crate::stmt::same_path(m, base) && crate::stmt::same_path(k, &indices[0])
                });
                Code::primary(cell.map(|f| f.2.clone()).unwrap_or_default())
            }
            ExprKind::Index(base, indices) if let Some(operands) = concat_at(base, indices) => {
                self.need("list_concat_at");
                let is_bool = matches!(&base.ty, Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) if **elem == Ty::Bool);
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
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..))
                    && indices.len() == 1
                    && self.map_slot(&indices[0]).is_some() =>
            {
                let at = self.map_slot(&indices[0]).unwrap_or_default();
                Code::primary(format!("cpl::map_slot_at({at})"))
            }
            ExprKind::Index(base, indices) => {
                let mut operands: Vec<&Expr> = vec![base];
                operands.extend(indices.iter());
                let base_ty = &base.ty;
                let raw_list = matches!(base.kind, ExprKind::Global(id) if self.program.raw_lists.contains(&id))
                    || matches!(indices[0].kind, ExprKind::Cast(Cast::Raw, _))
                    || self.inside_at(crate::bounds::var_of(&base.kind), indices)
                    || (self.target.checks == crate::Checks::None
                        && !matches!(base_ty, Ty::Range(_)));
                let checked = !proven(indices) && !raw_list;
                if !matches!(base_ty, Ty::Map(..) | Ty::HashMap(..)) {
                    let list = matches!(
                        base_ty,
                        Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..)
                    );
                    let raw = matches!(base.kind, ExprKind::Global(id) if self.program.raw_lists.contains(&id))
                        || matches!(indices[0].kind, ExprKind::Cast(Cast::Raw, _));
                    self.note_index(e.span, if list { !raw_list } else { checked }, indices, raw);
                }
                let joined = match base_ty {
                    Ty::Array { dims, .. }
                        if !checked
                            && matches!(base.kind, ExprKind::Local(_) | ExprKind::Global(_)) =>
                    {
                        crate::flat::joined(dims, indices)
                    }
                    _ => None,
                };
                if let Some(groups) = &joined {
                    operands = std::iter::once(&**base)
                        .chain(groups.iter().map(|g| g.0))
                        .collect();
                }
                let spans: Option<Vec<usize>> = joined.map(|g| g.iter().map(|g| g.1).collect());
                if (matches!(
                    base_ty,
                    Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..)
                ) && !raw_list)
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
                            let cell = match &spans {
                                Some(spans) => crate::flat::cell(&b, &at, spans),
                                None => format!("{b}{call}({})", at.join(", ")),
                            };
                            if **elem == Ty::Bool {
                                Code::new(format!("(bool){cell}"), UNARY)
                            } else {
                                Code::primary(cell)
                            }
                        }
                        Ty::List(elem)
                        | Ty::ListView(elem, _)
                        | Ty::Keyed(elem, _)
                        | Ty::MutView(elem, _)
                            if raw_list && **elem == Ty::Bool =>
                        {
                            Code::new(format!("(bool){b}[{}]", at[0]), UNARY)
                        }
                        Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..)
                            if raw_list =>
                        {
                            Code::primary(format!("{b}[{}]", at[0]))
                        }
                        Ty::List(elem)
                        | Ty::ListView(elem, _)
                        | Ty::Keyed(elem, _)
                        | Ty::MutView(elem, _)
                            if **elem == Ty::Bool =>
                        {
                            Code::new(
                                format!("(bool)cpl::at({}, {})", codes[0].text, at[0]),
                                UNARY,
                            )
                        }
                        Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..) => {
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
                        Ty::Str | Ty::StrView => {
                            Code::new(format!("(cpl::u8){b}[{}]", at[0]), UNARY)
                        }
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
                        Some(cpl_ir::NarrowLayout::OptOptI32) => {
                            Code::primary(format!("std::optional<cpl::OptInt>({})", code.text))
                        }
                        Some(cpl_ir::NarrowLayout::Tuple(_)) => {
                            let wide = wide_elem_ty.as_deref().unwrap_or("auto");
                            Code::primary(format!("{wide}({})", code.text))
                        }
                        _ => code,
                    }
                })
            }
            ExprKind::Field(base, index)
                if matches!(base.ty, Ty::Struct(id) if self.devirt.dropped_field(id, *index)
                    && e.ty == Ty::Unit) =>
            {
                Code::primary("(void)0")
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
                        if self.lent_fields.is_lent(*id, *index) {
                            Code::primary(format!("(*{access})"))
                        } else if self.rows.contains_key(&(*id, *index)) {
                            Code::primary(format!("{access}.vec()"))
                        } else if self.narrow_ints.contains_key(&(*id, *index)) {
                            Code::primary(self.narrow_field_load(*id, *index, &access))
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
                            .map(|idx| struct_id.is_some_and(|s| self.devirt.dropped_field(s, idx)))
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
                                if let Some(lent) = self.lent_item(struct_id, idx, item) {
                                    return lent;
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
                                    let v = self.expr_with_elem_layout(item, Some(&l)).text;
                                    return self.taken_item(item, v);
                                }
                                let v = self.braced(item);
                                let v = self.taken_item(item, v);
                                match struct_id {
                                    Some(s) => self.narrow_field_store(s, idx, &v),
                                    None => v,
                                }
                            })
                            .collect();
                        let cell = ty
                            .strip_prefix("std::vector<")
                            .and_then(|c| c.strip_suffix('>'));
                        if let Some(cell) = cell.filter(|c| c.contains("std::") && items.len() > 1)
                        {
                            // `std::vector{a, b}` copies each list out of its
                            // initializer list; this moves them.
                            self.need("list_of");
                            return Code::primary(format!(
                                "cpl::list_of<{cell}>({{{}}})",
                                items.join(", ")
                            ));
                        }
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
            ExprKind::IsSome(value) => Code::primary(self.has_value(value)),
            // The user's `!` is an assertion: on `none` it stops the
            // program, as it does in `cpl run`. After a test it is a plain read.
            ExprKind::Unwrap(value, checked) if crate::names::is_flag_opt(&value.ty) => {
                let code = self.expr(value);
                let flag = code.at(COND);
                Code::primary(if *checked {
                    format!("({flag} ? cpl::Unit() : (cpl::fail(\"`!` on `None`: there is no value here\"), cpl::Unit()))")
                } else {
                    format!("((void){flag}, cpl::Unit())")
                })
            }
            ExprKind::Unwrap(value, true) => {
                self.need("have");
                let code = self.expr(value);
                Code::primary(format!("cpl::have({})", code.text))
            }
            ExprKind::Unwrap(value, false) => {
                let code = self.expr(value);
                Code::new(format!("*{}", code.at(UNARY)), UNARY)
            }
            ExprKind::RangeCheck {
                value,
                lo,
                hi,
                text,
            } => {
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
                if let Some(leaves) = self.devirt.join_leaves(e) {
                    return self.pick(e, &leaves, c);
                }
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
                // Each closure is its own C++ type, so a join of fns other than
                // closures and fn locals meets at the fn's erased representation.
                let (a, b) = if matches!(e.ty, Ty::Fn(..)) {
                    let erased = self.ty(&e.ty);
                    let join = |x: Code| Code::primary(format!("{erased}({})", x.text));
                    (join(a), join(b))
                } else {
                    (a, b)
                };
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
                    return self
                        .packed_result(&e.ty, Code::primary(format!("cpl::map_take({p}, {k})")));
                }
                if let Some(place) = gives_its_place(block, &e.ty) {
                    return self.expr(place);
                }
                if let Some(leaves) = self.devirt.join_leaves(e) {
                    let index = self.join_index(e, &leaves);
                    return self.pick(e, &leaves, index);
                }
                let ret = if let Some(k) = self.expr_fixed_len(e) {
                    self.fixed_array_cpp(&e.ty, k)
                } else if gives_one_closure(block) {
                    // `plan.sorted(by: tree.pos)`: the lambda's own type, not a std::function.
                    "auto".to_string()
                } else {
                    self.ty_with_elem_layout(&e.ty, self.expr_elem_layout(e))
                };
                self.do_lambda(block, &e.ty, ret)
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
                if let Some(cid) = known.filter(|_| crate::devirt::direct_callee(callee)) {
                    self.direct.insert(cid);
                    let base = match &callee.kind {
                        ExprKind::Field(base, _) if !crate::devirt::skippable(callee) => {
                            Some(&**base)
                        }
                        _ => None,
                    };
                    let mut operands: Vec<&Expr> = base.into_iter().collect();
                    operands.extend(values);
                    return self.sequence(&operands, &e.ty, |this, mut codes| {
                        let base = base.map(|_| codes.remove(0).at(PRIMARY));
                        let ps = this.fn_value_args(args, codes);
                        let call = format!("{}({})", crate::devirt::name(cid), ps.join(", "));
                        Code::primary(match base {
                            Some(b) => format!("(static_cast<void>({b}), {call})"),
                            None => call,
                        })
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
                let d = dom_ident(program, *id);
                Code::primary(format!("cpl::DomainVal{{{d}_lo, {d}_len}}"))
            }
            ExprKind::Builtin(Builtin::Copy, args)
                if self.moved_entry.remove(&(e as *const _ as usize)) =>
            {
                self.moved_entry_code(&args[0])
            }
            ExprKind::Builtin(Builtin::Len, args) if matches!(&args[0].kind, ExprKind::Call(f, _) if self.pushers.contains_key(f)) => {
                self.count_pushed(&args[0])
            }
            ExprKind::Intrinsic(p, args) => {
                let operands: Vec<&Expr> = args.iter().collect();
                let ty = match args.first().map(|a| &a.ty) {
                    Some(Ty::Float) => "F64",
                    Some(Ty::I128) => "I128",
                    Some(Ty::U64) => "U64",
                    _ => "Int",
                };
                self.sequence(&operands, &e.ty, |this, codes| {
                    if let Some(need) = p.form(ty).need {
                        this.need(need);
                    }
                    let texts: Vec<String> = codes.into_iter().map(|c| c.text).collect();
                    Code::new(p.cpp(ty, &texts), crate::emit::UNARY)
                })
            }
            ExprKind::Builtin(op, args) => {
                let operands: Vec<&Expr> = args.iter().collect();
                if matches!(op, Builtin::ListConcat | Builtin::Copy) {
                    self.mark_read_in_place(&operands, None, &[]);
                }
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
                        let t = crate::emit::Touch::of_expr(a);
                        t.calls_code() || t.mutates
                    });
                let narrow_elem = self.place_elem_layout(place).cloned();
                let emit_mutate = |this: &mut Self, p: String, arg: String, arg2: String| -> Code {
                    match op {
                        Mutation::Push if place.ty == Ty::Str => {
                            Code::primary(format!("{p}.push_back({})", cast("char", &arg)))
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
                        Mutation::Insert if matches!(place.ty, Ty::List(_)) => {
                            this.need("list_insert");
                            let v = match &narrow_elem {
                                Some(cpl_ir::NarrowLayout::Seq(inner)) => {
                                    this.expr_with_elem_layout(&args[1], Some(inner)).text
                                }
                                Some(l) => this.narrow_cell_value(&arg2, l),
                                None => arg2,
                            };
                            Code::primary(format!("cpl::list_insert({p}, {arg}, {v})"))
                        }
                        Mutation::Remove if matches!(place.ty, Ty::List(_)) => {
                            this.need("list_remove");
                            Code::primary(format!("cpl::list_remove({p}, {arg})"))
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
                        Mutation::Resize if place.ty == Ty::Str => Code::primary(format!(
                            "{p}.resize((size_t)({arg}), {})",
                            cast("char", &arg2)
                        )),
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
                            let borrow = crate::uses::extend_borrows(place, &args[0]);
                            out.push_str(&Self::extend_source(
                                &args[0],
                                &src,
                                &arg,
                                this.indent,
                                borrow,
                            ));
                            out.push_str(&format!("{}auto& {r} = {p};\n", pad(this.indent)));
                            out.push_str(&format!(
                                "{}{r}.insert({r}.end(), std::begin({src}), std::end({src}));\n",
                                pad(this.indent)
                            ));
                        }),
                        Mutation::Fill if place.ty == Ty::Str => Code::primary(format!(
                            "std::fill({p}.begin(), {p}.end(), {})",
                            cast("char", &arg)
                        )),
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
                    let arg = args.first().map(|arg| self.stored(arg)).unwrap_or_default();
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
            ExprKind::MutLine(place, at, axis) => {
                self.need("grid_line");
                let p = self.place(place);
                let at = self.expr(at).text;
                Code::primary(format!("cpl::mut_line({p}, {at}, {axis})"))
            }
            ExprKind::Sort(place, sort_by, desc) => self.lambda("void", |this, out| {
                this.sort_stmts(place, sort_by, desc.as_deref(), out)
            }),
            ExprKind::Read(spec) => self.read(spec),
            // A bad token throws instead of stopping the program while the
            // string is the input; see `parse_str` in the runtime. (R10.2.4)
            ExprKind::ReadFrom(text, body) => {
                self.need("parse_str");
                let ty = self.ty(&e.ty);
                let text = self.expr(text).text;
                self.lambda(&ty, |this, out| {
                    let lead = pad(this.indent);
                    out.push_str(&format!("{lead}cpl::parse_scope cpl_scope({text});\n"));
                    out.push_str(&format!("{lead}try {{\n"));
                    this.indent += 1;
                    let value = this.expr(body).text;
                    this.indent -= 1;
                    out.push_str(&format!("{lead}    auto cpl_value = {value};\n"));
                    out.push_str(&format!(
                        "{lead}    if (cpl_scope.whole()) return {ty}(std::move(cpl_value));\n"
                    ));
                    out.push_str(&format!("{lead}}} catch (const cpl::parse_failed&) {{}}\n"));
                    out.push_str(&format!("{lead}return {ty}();\n"));
                })
            }
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
                        let calls = !touch.calls.is_empty();
                        if touch.io || !touch.writes.is_empty() || !touch.binds.is_empty() || calls
                        {
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
                if let Some(k) = self.ty_fixed_len(&e.ty) {
                    return self.expr_with_fixed_len(e, Some(k));
                }
                if let Some(zeros) = self.zeros(&cell, dims, value.as_deref()) {
                    zeros
                } else if dims.len() == 1 {
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
            ExprKind::DomainLen(d) => Code::primary(format!("{}_len", dom_ident(program, *d))),
            ExprKind::DomainLo(d) => Code::primary(format!("{}_lo", dom_ident(program, *d))),
        }
    }
}

/// True when no index is a bare `Int`: each is a position of the
/// dimension's domain, or an `Int` the writer vouched for in `unchecked:`.
pub(crate) fn proven(indices: &[Expr]) -> bool {
    indices.iter().all(|index| {
        index.ty != Ty::Int || matches!(index.kind, ExprKind::Cast(Cast::Raw | Cast::Trust, _))
    })
}

/// Recognizes the IR `Do` block emitted for `m.remove(k)` on a `Map` or `HashMap`:
/// `[...settled_indexes, let key = k, let held = map_get(m, key), if remove(m, key) {}, give held]`.
pub(crate) fn map_remove_do(e: &Expr) -> Option<(&[Stmt], &Place, &Expr)> {
    let ExprKind::Do(block) = &e.kind else {
        return None;
    };
    let n = block.stmts.len();
    if n < 4 {
        return None;
    }
    let (prefix, tail) = block.stmts.split_at(n - 4);
    let [Stmt::Let(key_id, Some(key_val)), Stmt::Let(
        held_id,
        Some(Expr {
            kind: ExprKind::Builtin(Builtin::MapGet, get_args),
            ..
        }),
    ), Stmt::If(
        Expr {
            kind: ExprKind::Mutate(Mutation::Remove, place, rem_args),
            ..
        },
        then_b,
        else_b,
    ), Stmt::Give(Expr {
        kind: ExprKind::Local(give_id),
        ..
    })] = tail
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

impl Emitter<'_> {
    /// `cpl::pick(index, leaf0, leaf1, ..)`: a value `if` or `match` of
    /// closures (see [`crate::devirt::Devirt::join_leaves`]) keeps each in
    /// its own type and calls the one `index` names. For an `if`, `c` is
    /// its condition, already written.
    fn pick(&mut self, e: &Expr, leaves: &[&Expr], c: Code) -> Code {
        self.need("pick");
        let index = match &e.kind {
            ExprKind::If(_, a, b) => {
                let a = self.join_index(a, leaves);
                let b = self.join_index(b, leaves);
                format!("{} ? {} : {}", c.at(COND + 1), a.text, b.text)
            }
            _ => c.text,
        };
        let mut args = vec![index];
        args.extend(leaves.iter().map(|leaf| self.expr(leaf).text));
        Code::primary(format!("cpl::pick({})", args.join(", ")))
    }

    /// The index into `leaves` that join `e` picks, as an int expression.
    fn join_index(&mut self, e: &Expr, leaves: &[&Expr]) -> Code {
        if let Some(k) = leaves.iter().position(|leaf| std::ptr::eq(*leaf, e)) {
            return Code::primary(k.to_string());
        }
        match &e.kind {
            ExprKind::If(cond, a, b) => {
                let c = self.expr(cond);
                let a = self.join_index(a, leaves);
                let b = self.join_index(b, leaves);
                Code {
                    text: format!("{} ? {} : {}", c.at(COND + 1), a.at(COND + 1), b.at(COND)),
                    prec: COND,
                    narrow: false,
                }
            }
            ExprKind::Do(block) => {
                let indices: BTreeMap<usize, usize> = leaves
                    .iter()
                    .enumerate()
                    .map(|(k, leaf)| (*leaf as *const Expr as usize, k))
                    .collect();
                let outer = std::mem::replace(&mut self.give_index, indices);
                let give_ref = std::mem::replace(&mut self.give_ref, false);
                let give_owned = self.give_owned.take();
                let code = self.lambda("int", |this, out| this.block(block, out));
                self.give_index = outer;
                self.give_ref = give_ref;
                self.give_owned = give_owned;
                code
            }
            _ => unreachable!("join_leaves gives only ifs, dos and their leaves"),
        }
    }
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
