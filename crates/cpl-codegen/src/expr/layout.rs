//! Values built to a layout: fill dimensions, element layouts, fixed lengths and packed results.

use super::*;

impl<'p> Emitter<'p> {
    pub fn dim(&mut self, dim: &DimSpec) -> String {
        match dim {
            DimSpec::Pos(d) => format!("{}_len", dom_ident(self.program, *d)),
            DimSpec::Bound(d) => format!("{}_len + 1", dom_ident(self.program, *d)),
            DimSpec::Sym(d) => {
                let dn = dom_ident(self.program, *d);
                format!("({dn}_len * ({dn}_len + 1) / 2)")
            }
            DimSpec::Enum(e) => self.program.enums[*e].variants.len().to_string(),
            DimSpec::Int(len) | DimSpec::IntFrom(_, len) => self.expr(len).text,
        }
    }

    /// Whether `dims` of `cell`s filled with `value` are all zero bytes and may
    /// span pages, so calloc should make them.
    pub fn zero_fill(&mut self, cell: &str, dims: &[DimSpec], value: Option<&Expr>) -> bool {
        let value = value.map(|v| match &v.kind {
            ExprKind::RangeCheck { value, .. } => value,
            _ => v,
        });
        let zero = value.is_none_or(|v| zero_bits(self.program, v, &[], 0));
        let plain = cell_bytes(cell).or_else(|| value.and_then(|v| self.plain_bytes(&v.ty)));
        let bytes: i64 = match plain {
            Some(b) => b,
            None => return false,
        };
        let len = |d: &DimSpec| match d {
            DimSpec::Pos(d) => self.facts.domain_max_len(*d),
            DimSpec::Bound(d) => self.facts.domain_max_len(*d).map(|n| n + 1),
            DimSpec::Sym(d) => self.facts.domain_max_len(*d).map(|n| n * (n + 1) / 2),
            DimSpec::Enum(_) => None,
            DimSpec::Int(len) | DimSpec::IntFrom(_, len) => match len.kind {
                ExprKind::Int(n) => Some(n),
                _ => None,
            },
        };
        // Tables under malloc's mmap threshold come from its heap, where calloc
        // writes them anyway; an unbounded length counts as one.
        let big = dims
            .iter()
            .map(|d| len(d).unwrap_or(1))
            .fold(bytes, i64::saturating_mul)
            >= 1 << 18;
        zero && big && {
            self.need("zeros");
            true
        }
    }

    /// `dims` of cells nothing has written yet, for a table comprehension
    /// to write (see [`Emitter::filled_next`]).
    pub fn blank(&mut self, cell: &str, dims: &[DimSpec]) -> Code {
        let lens: Vec<String> = dims.iter().map(|d| self.dim(d)).collect();
        Code::primary(match &lens[..] {
            [n] => {
                self.need("blank");
                format!("cpl::blank<{cell}>({n})")
            }
            _ => {
                self.need("blank_grid");
                format!(
                    "cpl::blank_grid<{cell}, {}>({{{}}})",
                    lens.len(),
                    lens.join(", ")
                )
            }
        })
    }

    /// `dims` of `value` from calloc, when every cell is zero bytes.
    pub fn zeros(&mut self, cell: &str, dims: &[DimSpec], value: Option<&Expr>) -> Option<Code> {
        if !self.zero_fill(cell, dims, value) {
            return None;
        }
        let lens: Vec<String> = dims.iter().map(|d| self.dim(d)).collect();
        if lens.len() > 1 {
            self.need("zero_grid");
        }
        let zeros = match &lens[..] {
            [n] => format!("cpl::zeros<{cell}>({n})"),
            _ => format!(
                "cpl::zero_grid<{cell}, {}>({{{}}})",
                lens.len(),
                lens.join(", ")
            ),
        };
        // A checked zero keeps its check.
        Some(Code::primary(match value {
            Some(
                v @ Expr {
                    kind: ExprKind::RangeCheck { .. },
                    ..
                },
            ) => {
                format!("((void){}, {zeros})", self.expr(v).text)
            }
            _ => zeros,
        }))
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
            Some(v) => format!("{n}, {}", self.fill_value(v, layout)),
            None => n,
        }
    }

    /// A filled array's cell value, as the array's cells hold it.
    pub fn fill_value(&mut self, v: &Expr, layout: Option<&cpl_ir::NarrowLayout>) -> String {
        match layout {
            Some(cpl_ir::NarrowLayout::Seq(inner)) => {
                self.expr_with_elem_layout(v, Some(inner)).text
            }
            Some(l) => {
                let text = self.expr(v).text;
                self.narrow_cell_value(&text, l)
            }
            None => self.stored(v),
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
        if matches!(l, cpl_ir::NarrowLayout::Map(..)) {
            return match &e.kind {
                ExprKind::Tuple(items) if items.is_empty() => {
                    Code::primary(format!("{}()", self.ty_with_elem_layout(&e.ty, Some(l))))
                }
                _ => self.expr(e),
            };
        }
        let elem_ty = match &e.ty {
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _) => Some(elem.as_ref()),
            _ => None,
        };
        let Some(cpp_cell) = elem_ty.and_then(|et| self.narrow_cell(l, et)) else {
            return self.expr(e);
        };
        match &e.kind {
            ExprKind::Fill(dims, value) => {
                if let Some(zeros) = self.zeros(&cpp_cell, dims, value.as_deref()) {
                    zeros
                } else if dims.len() == 1 {
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
                    Code::primary(format!(
                        "std::vector<{cpp_cell}>{{{}}}",
                        rendered.join(", ")
                    ))
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
                self.do_lambda(block, &e.ty, ret)
            }
            _ => self.expr(e),
        }
    }

    /// Whether optional `x` holds a value. `m.get(k)` finds the entry and
    /// copies nothing.
    pub(super) fn has_value(&mut self, x: &Expr) -> String {
        if let ExprKind::Local(s) = x.kind {
            if self.opened.contains_key(&s) {
                return self
                    .expr(&Expr {
                        kind: ExprKind::IsSome(Box::new(x.clone())),
                        ty: Ty::Bool,
                        span: x.span,
                    })
                    .text;
            }
        }
        if let ExprKind::Builtin(Builtin::MapGet, got) = &x.kind {
            self.need("map_lend");
            let (m, k) = (self.expr(&got[0]), self.expr(&got[1]));
            return format!(
                "cpl::map_lend({}, {}).has_value()",
                m.at(PRIMARY),
                k.at(PRIMARY)
            );
        }
        if crate::names::is_flag_opt(&x.ty) {
            return self.expr(x).at(PRIMARY);
        }
        format!("{}.has_value()", self.expr(x).at(PRIMARY))
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
    pub(super) fn packed_result(&mut self, ty: &Ty, code: Code) -> Code {
        let cpp = self.ty(ty);
        match ty {
            Ty::Opt(_) if cpp.starts_with("cpl::Opt") => {
                Code::primary(format!("{cpp}({})", code.text))
            }
            _ => code,
        }
    }
}

/// The bytes of a number cell, which holds whatever bytes it is given.
impl Emitter<'_> {
    /// The bytes of a struct whose fields are all numbers, at any depth.
    fn plain_bytes(&self, ty: &Ty) -> Option<i64> {
        let Ty::Struct(s) = ty else {
            return cell_bytes(&elem_ty(self.program, self.niches, ty));
        };
        let def = &self.program.structs[*s];
        if !def.fixed.is_empty() || def.fields.is_empty() {
            return None;
        }
        def.fields.iter().map(|(_, t)| self.plain_bytes(t)).sum()
    }
}

/// `e` is a value whose every byte is zero: a zero number, a struct of them,
/// or a call of a fn whose one expression makes one from zero arguments
/// (`Mod(x % P)` for `x = 0`). `zeros` marks the locals known to be zero.
fn zero_bits(program: &cpl_ir::Program, e: &Expr, zeros: &[bool], depth: usize) -> bool {
    let zero = |x: &Expr| zero_bits(program, x, zeros, depth);
    match &e.kind {
        ExprKind::Int(0) | ExprKind::Bool(false) | ExprKind::Char(0) => true,
        ExprKind::Float(f) => f.to_bits() == 0,
        ExprKind::Local(s) => zeros.get(*s).copied().unwrap_or(false),
        ExprKind::Cast(Cast::IntToFloat | Cast::IntToI128 | Cast::IntToU64, x) => zero(x),
        ExprKind::Tuple(items) if matches!(e.ty, Ty::Struct(_)) => {
            !items.is_empty() && items.iter().all(zero)
        }
        ExprKind::Binary(op, l, r) => match op {
            BinOp::Mul | BinOp::BitAnd => zero(l) || zero(r),
            BinOp::Add | BinOp::Sub | BinOp::BitOr | BinOp::BitXor => zero(l) && zero(r),
            BinOp::Div | BinOp::Rem => zero(l) && matches!(r.kind, ExprKind::Int(n) if n != 0),
            BinOp::Shl | BinOp::Shr => zero(l),
            _ => false,
        },
        ExprKind::Call(f, args) if depth < 4 => {
            let callee = &program.fns[*f];
            let [Stmt::Return(Some(body)) | Stmt::Give(body) | Stmt::Expr(body)] =
                callee.body.stmts.as_slice()
            else {
                return false;
            };
            if args.len() != callee.params.len() {
                return false;
            }
            let mut known = vec![false; callee.locals.len()];
            for (slot, arg) in args.iter().enumerate() {
                match arg {
                    Arg::Value(v) => known[slot] = zero(v),
                    _ => return false,
                }
            }
            zero_bits(program, body, &known, depth + 1)
        }
        _ => false,
    }
}

fn cell_bytes(cell: &str) -> Option<i64> {
    cpl_ir::IntWidth::cpp_bytes(cell)
}

/// A cell C++ leaves unset until written: a number.
pub(crate) fn plain_cell(cell: &str) -> bool {
    cell_bytes(cell).is_some()
}
