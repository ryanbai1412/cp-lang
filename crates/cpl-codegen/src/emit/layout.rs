//! C++ types and element layouts: narrow cells, fixed arrays, type definitions.

use super::*;

impl<'p> Emitter<'p> {
    pub fn current_scope(&self) -> cpl_ir::ScopeId {
        if let Some(cid) = self.closure_id {
            cpl_ir::ScopeId::Closure(cid)
        } else if let Some(fid) = self.fn_id {
            cpl_ir::ScopeId::Fn(fid)
        } else {
            cpl_ir::ScopeId::Main
        }
    }

    pub fn elem_layout(&self, loc: cpl_ir::ElemLoc) -> Option<&cpl_ir::NarrowLayout> {
        self.narrow_elems.get(&loc)
    }

    pub fn expr_elem_layout(&self, e: &cpl_ir::Expr) -> Option<&cpl_ir::NarrowLayout> {
        use cpl_ir::{Builtin, ElemLoc, ExprKind, NarrowLayout, Stmt};
        let scope = self.current_scope();
        match &e.kind {
            ExprKind::Global(g) => self.elem_layout(ElemLoc::Global(*g)),
            ExprKind::Local(s) => self.elem_layout(ElemLoc::Local(scope, *s)),
            ExprKind::Field(base, f) => match base.ty {
                Ty::Struct(id) => self.elem_layout(ElemLoc::Field(id, *f)),
                _ => None,
            },
            ExprKind::Call(fid, _) => self.elem_layout(ElemLoc::FnRet(*fid)),
            ExprKind::Index(base, _) => match self.expr_elem_layout(base) {
                Some(NarrowLayout::Seq(inner)) => Some(inner.as_ref()),
                _ => None,
            },
            ExprKind::Builtin(
                Builtin::ListSlice
                | Builtin::KeySlice
                | Builtin::Rekey
                | Builtin::ToList
                | Builtin::Copy,
                args,
            ) => args.first().and_then(|a| self.expr_elem_layout(a)),
            ExprKind::Do(blk) => match blk.stmts.last() {
                Some(Stmt::Give(inner)) => self.expr_elem_layout(inner),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn place_elem_layout(&self, p: &cpl_ir::Place) -> Option<&cpl_ir::NarrowLayout> {
        use cpl_ir::{ElemLoc, NarrowLayout, PlaceKind};
        let scope = self.current_scope();
        match &p.kind {
            PlaceKind::Global(g) => self.elem_layout(ElemLoc::Global(*g)),
            PlaceKind::Local(s) => self.elem_layout(ElemLoc::Local(scope, *s)),
            PlaceKind::Field(base, f) => match base.ty {
                Ty::Struct(id) => self.elem_layout(ElemLoc::Field(id, *f)),
                _ => None,
            },
            PlaceKind::Index(base, _) => match self.place_elem_layout(base) {
                Some(NarrowLayout::Seq(inner)) => Some(inner.as_ref()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn fixed_array_len(&self, loc: cpl_ir::ElemLoc) -> Option<u32> {
        self.fixed_arrays
            .iter()
            .find(|(l, _)| *l == loc)
            .map(|(_, k)| *k as u32)
    }

    pub fn place_fixed_len(&self, p: &cpl_ir::Place) -> Option<u32> {
        let scope = self.current_scope();
        match &p.kind {
            cpl_ir::PlaceKind::Global(g) => self.fixed_array_len(cpl_ir::ElemLoc::Global(*g)),
            cpl_ir::PlaceKind::Local(s) => self.fixed_array_len(cpl_ir::ElemLoc::Local(scope, *s)),
            _ => None,
        }
    }

    pub fn expr_fixed_len(&self, e: &cpl_ir::Expr) -> Option<u32> {
        let scope = self.current_scope();
        match &e.kind {
            cpl_ir::ExprKind::Global(g) => self.fixed_array_len(cpl_ir::ElemLoc::Global(*g)),
            cpl_ir::ExprKind::Local(s) => self.fixed_array_len(cpl_ir::ElemLoc::Local(scope, *s)),
            cpl_ir::ExprKind::Do(b) => match b.stmts.last() {
                Some(cpl_ir::Stmt::Give(inner)) => self.expr_fixed_len(inner),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn fixed_array_cpp(&self, ty: &Ty, k: u32) -> String {
        if let Ty::Array { elem, .. } = ty {
            let cell = crate::names::elem_ty(self.program, self.niches, elem);
            return format!("std::array<{cell}, {k}>");
        }
        self.ty(ty)
    }

    pub fn ty_for_loc(&self, ty: &Ty, loc: cpl_ir::ElemLoc) -> String {
        if let Some(k) = self.fixed_array_len(loc) {
            return self.fixed_array_cpp(ty, k);
        }
        self.ty_with_elem_layout(ty, self.elem_layout(loc))
    }

    pub fn ty_with_elem_layout(&self, ty: &Ty, layout: Option<&cpl_ir::NarrowLayout>) -> String {
        if let Some(l) = layout {
            let mut leaf = l;
            while let cpl_ir::NarrowLayout::Seq(inner) = leaf {
                leaf = inner;
            }
            if *leaf == cpl_ir::NarrowLayout::OptI32 {
                self.niche_types.set(self.niche_types.get() | 1 << 6);
            }
            match ty {
                Ty::Array { dims, elem } => {
                    if let Some(cell) = l.cpp_elem(elem) {
                        return if dims.len() == 1 {
                            format!("std::vector<{cell}>")
                        } else {
                            format!("cpl::Grid<{cell}, {}>", dims.len())
                        };
                    }
                }
                Ty::List(elem) => {
                    if let Some(cell) = l.cpp_elem(elem) {
                        return format!("std::vector<{cell}>");
                    }
                }
                Ty::ListView(elem, _) => {
                    if let Some(cell) = l.cpp_elem(elem) {
                        return format!("cpl::ListView<{cell}>");
                    }
                }
                Ty::Keyed(elem, _) => {
                    if let Some(cell) = l.cpp_elem(elem) {
                        return format!("cpl::Keyed<{cell}>");
                    }
                }
                Ty::MutView(elem, _) => {
                    if let Some(cell) = l.cpp_elem(elem) {
                        return format!("cpl::MutView<{cell}>");
                    }
                }
                _ => {}
            }
        }
        self.ty(ty)
    }

    pub fn narrow_cell_value(&self, text: &str, layout: &cpl_ir::NarrowLayout) -> String {
        match layout {
            cpl_ir::NarrowLayout::Int(w) => format!("{}({text})", w.cpp()),
            cpl_ir::NarrowLayout::OptI32 => format!("cpl::OptI32({text})"),
            cpl_ir::NarrowLayout::Tuple(ws) => {
                let parts: Vec<&str> = ws.iter().map(|w| w.cpp()).collect();
                format!("std::tuple<{}>({text})", parts.join(", "))
            }
            cpl_ir::NarrowLayout::Seq(_) => text.to_string(),
        }
    }

    /// The initializer that gives a new variable its type's zero.
    pub fn zero_init(&self, ty: &Ty) -> &'static str {
        match ty {
            Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Char | Ty::Pos(_) | Ty::Bound(_) => {
                " = 0"
            }
            Ty::Bool => " = false",
            Ty::Enum(id) if is_plain_enum(self.program, *id) => "{}",
            _ => "",
        }
    }

    /// The types `ty` holds by value, which must be complete before it.
    pub(super) fn needs_complete(&self, ty: &Ty, out: &mut Vec<Ty>) {
        match ty {
            Ty::Struct(_) | Ty::Enum(_) => out.push(ty.clone()),
            Ty::Opt(inner)
            | Ty::List(inner)
            | Ty::Set(inner)
            | Ty::HashSet(inner)
            | Ty::Range(inner) => self.needs_complete(inner, out),
            Ty::Array { elem, .. } => self.needs_complete(elem, out),
            Ty::Map(key, value) | Ty::HashMap(key, value) => {
                self.needs_complete(key, out);
                self.needs_complete(value, out);
            }
            Ty::Fn(args, ret) => {
                args.iter().for_each(|t| self.needs_complete(t, out));
                self.needs_complete(ret, out);
            }
            Ty::Tuple(items) => items.iter().for_each(|t| self.needs_complete(t, out)),
            Ty::Record(fields) => fields.iter().for_each(|(_, t)| self.needs_complete(t, out)),
            _ => {}
        }
    }

    pub(super) fn type_def(&mut self, ty: &Ty, done: &mut BTreeSet<Ty>, out: &mut String) {
        if !done.insert(ty.clone()) {
            return;
        }
        let program = self.program;
        let mut deps = Vec::new();
        match ty {
            Ty::Struct(id) => program.structs[*id]
                .fields
                .iter()
                .for_each(|(_, t)| self.needs_complete(t, &mut deps)),
            Ty::Enum(id) => {
                for (_, fields) in &program.enums[*id].variants {
                    fields
                        .iter()
                        .for_each(|t| self.needs_complete(t, &mut deps));
                }
            }
            _ => {}
        }
        for dep in deps {
            self.type_def(&dep, done, out);
        }
        match ty {
            Ty::Struct(id) => {
                let def = &program.structs[*id];
                out.push_str(&format!("struct {} {{\n", ident(&def.name)));
                let mut names = Vec::new();
                for (field_idx, (field, field_ty)) in def.fields.iter().enumerate() {
                    let name = ident(field);
                    if self.devirt.unread_field(*id, field_idx) {
                        continue;
                    }
                    if self.devirt.field_elidable(*id, field_idx) {
                        out.push_str(&format!("    bool {name} = false;\n"));
                        names.push(name);
                        continue;
                    }
                    if let (true, Ty::Fn(_, ret)) =
                        (self.devirt.unbuilt_field(*id, field_idx), field_ty)
                    {
                        self.need("no_fn");
                        let ret = self.ty(ret);
                        out.push_str(&format!("    cpl::NoFn<{ret}> {name};\n"));
                        names.push(name);
                        continue;
                    }
                    self.need_ty(field_ty);
                    let mut cpp = self.ty(field_ty);
                    let cells = self.rows.get(&(*id, field_idx));
                    if let (Some(k), Some(cell)) = (
                        cells,
                        cpp.strip_prefix("std::vector<")
                            .and_then(|c| c.strip_suffix('>')),
                    ) {
                        cpp = format!("cpl::Row<{cell}, {k}>");
                        self.need("row");
                    } else if let Some(w) = self.narrow_ints.get(&(*id, field_idx)) {
                        cpp = w.cpp().into();
                    } else if let Some(l) = self
                        .narrow_elems
                        .get(&cpl_ir::ElemLoc::Field(*id, field_idx))
                    {
                        cpp = self.ty_with_elem_layout(field_ty, Some(l));
                    }
                    out.push_str(&format!(
                        "    {} {}{};\n",
                        cpp,
                        name,
                        self.zero_init(field_ty)
                    ));
                    if !def.cached_fields.contains(&field_idx) {
                        names.push(name);
                    }
                }
                if Ty::Struct(*id).is_comparable(program) {
                    out.push_str(&format!(
                        "    auto cpl_tie() const {{ return std::tie({}); }}\n",
                        names.join(", ")
                    ));
                }
                out.push_str("};\n");
            }
            Ty::Enum(id) if is_plain_enum(program, *id) => {
                let def = &program.enums[*id];
                let names: Vec<String> = def.variants.iter().map(|(v, _)| ident(v)).collect();
                out.push_str(&format!(
                    "enum class {} {{ {} }};\n",
                    ident(&def.name),
                    names.join(", ")
                ));
            }
            Ty::Enum(id) => {
                let def = &program.enums[*id];
                let name = ident(&def.name);
                let tags: Vec<String> = def.variants.iter().map(|(v, _)| ident(v)).collect();
                out.push_str(&format!(
                    "struct {name} {{\n    enum Tag {{ {} }} tag = {};\n",
                    tags.join(", "),
                    tags[0]
                ));
                let mut tie = vec!["tag".to_string()];
                let mut makers = String::new();
                for (variant, fields) in &def.variants {
                    let variant = ident(variant);
                    let mut params = Vec::new();
                    let mut sets = String::new();
                    for (i, field_ty) in fields.iter().enumerate() {
                        let field = format!("{variant}_{i}");
                        let cpp = self.ty(field_ty);
                        out.push_str(&format!("    {cpp} {field}{};\n", self.zero_init(field_ty)));
                        params.push(if field_ty.is_scalar() {
                            format!("{cpp} a{i}")
                        } else {
                            format!("const {cpp}& a{i}")
                        });
                        sets.push_str(&format!(" v.{field} = a{i};"));
                        tie.push(field);
                    }
                    if !fields.is_empty() {
                        makers.push_str(&format!(
                            "    static {name} make_{variant}({}) {{ {name} v; v.tag = {variant};{sets} return v; }}\n",
                            params.join(", ")
                        ));
                    } else {
                        makers.push_str(&format!("    static {name} make_{variant}() {{ {name} v; v.tag = {variant}; return v; }}\n"));
                    }
                }
                out.push_str(&makers);
                out.push_str(&format!(
                    "    auto cpl_tie() const {{ return std::tie({}); }}\n}};\n",
                    tie.join(", ")
                ));
            }
            _ => {}
        }
    }
}
