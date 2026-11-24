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
                None => format!("{n}, {}", self.stored(v)),
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
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _) => Some(elem.as_ref()),
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
        if let ExprKind::Builtin(Builtin::MapGet, got) = &x.kind {
            self.need("map_lend");
            let (m, k) = (self.expr(&got[0]), self.expr(&got[1]));
            return format!(
                "cpl::map_lend({}, {}).has_value()",
                m.at(PRIMARY),
                k.at(PRIMARY)
            );
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
