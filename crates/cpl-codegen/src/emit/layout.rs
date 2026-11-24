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
                | Builtin::GridLine
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

    /// The cell count of an array whose type fixes it: a `std::array` wherever it is held.
    pub fn ty_fixed_len(&self, ty: &Ty) -> Option<u32> {
        match ty {
            Ty::Array { dims, .. } if dims.len() == 1 => {
                crate::names::fixed_len(self.program, &dims[0])
            }
            _ => None,
        }
    }

    pub fn place_fixed_len(&self, p: &cpl_ir::Place) -> Option<u32> {
        if let Some(k) = self.ty_fixed_len(&p.ty) {
            return Some(k);
        }
        let scope = self.current_scope();
        match &p.kind {
            cpl_ir::PlaceKind::Global(g) => self.fixed_array_len(cpl_ir::ElemLoc::Global(*g)),
            cpl_ir::PlaceKind::Local(s) => self.fixed_array_len(cpl_ir::ElemLoc::Local(scope, *s)),
            _ => None,
        }
    }

    pub fn expr_fixed_len(&self, e: &cpl_ir::Expr) -> Option<u32> {
        if let Some(k) = self.ty_fixed_len(&e.ty) {
            return Some(k);
        }
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

    /// The signature of the `cpl::FnRef` that holds `slot` of fn `id`
    /// ([`crate::uses::fn_refs`]), when it is one.
    pub fn fn_view(&self, id: cpl_ir::FnId, slot: usize) -> Option<String> {
        if !self.fn_refs[id].contains(&slot) {
            return None;
        }
        let fun = match &self.program.fns[id].locals[slot].ty {
            Ty::Opt(inner) => &**inner,
            ty => ty,
        };
        let cpp = self.ty(fun);
        let sig = cpp.strip_prefix("std::function<")?.strip_suffix('>')?;
        Some(sig.to_string())
    }

    pub fn ty_for_loc(&self, ty: &Ty, loc: cpl_ir::ElemLoc) -> String {
        if let Some(k) = self.fixed_array_len(loc) {
            if let (Ty::Array { elem, .. }, Some(l)) = (ty, self.elem_layout(loc)) {
                if let Some(cell) = self.narrow_cell(l, elem) {
                    return format!("std::array<{cell}, {k}>");
                }
            }
            return self.fixed_array_cpp(ty, k);
        }
        let cpp = self.ty_with_elem_layout(ty, self.elem_layout(loc));
        if let cpl_ir::ElemLoc::Local(cpl_ir::ScopeId::Fn(id), slot) = loc {
            if let Some(sig) = self.fn_view(id, slot) {
                return format!("cpl::FnRef<{sig}>");
            }
            if self.lent_opts[id].contains(&slot) {
                if let Some(held) = cpp
                    .strip_prefix("std::optional<")
                    .and_then(|t| t.strip_suffix('>'))
                {
                    return format!("cpl::Lent<{held}>");
                }
            }
        }
        cpp
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
            if *leaf == cpl_ir::NarrowLayout::OptOptI32 {
                self.niche_types.set(self.niche_types.get() | 1 << 8);
            }
            match ty {
                Ty::Array { dims, elem } => {
                    if let Some(cell) = self.narrow_cell(l, elem) {
                        return if let Some(k) = self.ty_fixed_len(ty) {
                            format!("std::array<{cell}, {k}>")
                        } else if dims.len() == 1 {
                            format!("std::vector<{cell}>")
                        } else {
                            format!("cpl::Grid<{cell}, {}>", dims.len())
                        };
                    }
                }
                Ty::List(elem) => {
                    if let Some(cell) = self.narrow_cell(l, elem) {
                        return format!("std::vector<{cell}>");
                    }
                }
                Ty::ListView(elem, _) => {
                    if let Some(cell) = self.narrow_cell(l, elem) {
                        return format!("cpl::ListView<{cell}>");
                    }
                }
                Ty::Keyed(elem, _) => {
                    if let Some(cell) = self.narrow_cell(l, elem) {
                        return format!("cpl::Keyed<{cell}>");
                    }
                }
                Ty::MutView(elem, _) => {
                    if let Some(cell) = self.narrow_cell(l, elem) {
                        return format!("cpl::MutView<{cell}>");
                    }
                }
                Ty::Map(key, value) | Ty::HashMap(key, value) => {
                    if let cpl_ir::NarrowLayout::Map(kw, vw) = l {
                        let part = |w: &Option<cpl_ir::IntWidth>, t: &Ty| match w {
                            Some(w) => w.cpp().to_string(),
                            None => crate::names::value_ty(self.program, self.niches, t),
                        };
                        let (k, v) = (part(kw, key), part(vw, value));
                        let hashed = matches!(ty, Ty::HashMap(..))
                            || self.program.unordered_maps.contains(key.as_ref());
                        return if hashed {
                            format!("cpl::HashMap<{k}, {v}, cpl::CpHash>")
                        } else {
                            format!("std::map<{k}, {v}>")
                        };
                    }
                }
                _ => {}
            }
        }
        self.ty(ty)
    }

    /// The C++ cell of a sequence with layout `l` holding `elem`s; a narrow map cell is a map type.
    pub(crate) fn narrow_cell(&self, l: &cpl_ir::NarrowLayout, elem: &Ty) -> Option<String> {
        match l {
            cpl_ir::NarrowLayout::Seq(inner)
                if matches!(
                    elem,
                    Ty::Map(..) | Ty::HashMap(..) | Ty::Array { .. } | Ty::List(_)
                ) && self.layout_cell(l, elem).is_none() =>
            {
                let cell = self.ty_with_elem_layout(elem, Some(inner));
                (cell != self.ty(elem)).then_some(cell)
            }
            _ => self.layout_cell(l, elem),
        }
    }

    /// The niche field `idx` of struct `sid` is held in when it is a narrowed `?Int` or `??Int`.
    fn narrow_opt_field(
        &self,
        sid: usize,
        idx: usize,
    ) -> Option<(&'static str, &'static str, u16)> {
        if !self.narrow_ints.contains_key(&(sid, idx)) {
            return None;
        }
        crate::scope::opt_int_niche(self.program.structs[sid].fields.get(idx).map(|(_, t)| t))
    }

    /// The C++ type of a narrowed field: one rule for `Int`, `?Int` and `??Int` (R03.2).
    pub fn narrow_field_cpp(&self, sid: usize, idx: usize, w: cpl_ir::IntWidth) -> String {
        if let Some((narrow, _, bit)) = self.narrow_opt_field(sid, idx) {
            self.niche_types.set(self.niche_types.get() | 1 << bit);
            return narrow.into();
        }
        w.cpp().into()
    }

    /// `v` stored into a narrowed field.
    pub fn narrow_field_store(&self, sid: usize, idx: usize, v: &str) -> String {
        match self.narrow_ints.get(&(sid, idx)) {
            Some(w) => format!("{}({v})", self.narrow_field_cpp(sid, idx, *w)),
            None => v.to_string(),
        }
    }

    /// A narrowed field read back at its full width.
    pub fn narrow_field_load(&self, sid: usize, idx: usize, access: &str) -> String {
        if let Some((_, wide, _)) = self.narrow_opt_field(sid, idx) {
            format!("{wide}({access})")
        } else {
            format!("ll({access})")
        }
    }

    pub fn narrow_cell_value(&self, text: &str, layout: &cpl_ir::NarrowLayout) -> String {
        match layout {
            cpl_ir::NarrowLayout::Int(w) => format!("{}({text})", w.cpp()),
            cpl_ir::NarrowLayout::OptI32 => format!("cpl::OptI32({text})"),
            cpl_ir::NarrowLayout::OptOptI32 => format!("cpl::OptOptI32({text})"),
            cpl_ir::NarrowLayout::Tuple(fs) => {
                let kept = |t: &Ty| crate::names::value_ty(self.program, self.niches, t);
                let parts: Vec<String> = fs.iter().map(|f| f.cpp(&kept)).collect();
                format!("std::tuple<{}>({text})", parts.join(", "))
            }
            cpl_ir::NarrowLayout::Seq(_) | cpl_ir::NarrowLayout::Map(..) => text.to_string(),
        }
    }

    /// The C++ cell of a container of `elem_ty` stored at `layout`.
    pub fn layout_cell(&self, layout: &cpl_ir::NarrowLayout, elem_ty: &Ty) -> Option<String> {
        layout.cpp_elem(elem_ty, &|t| {
            crate::names::value_ty(self.program, self.niches, t)
        })
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
                if !crate::names::fn_pointer(self.program, args, ret) {
                    for t in crate::names::fn_bound(self.program, args, ret).unwrap_or_default() {
                        self.needs_complete(&t, out);
                    }
                }
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
                for idx in 0..def.fields.len() {
                    let Some(cid) = self.devirt.field_functor(*id, idx) else {
                        continue;
                    };
                    if !self.functors.insert(cid) {
                        continue;
                    }
                    for cap in &program.closures[cid].captures {
                        let ty = &program.closures[cid].locals[cap.inner].ty;
                        if matches!(ty, Ty::Struct(_)) {
                            self.type_def(ty, done, out);
                        }
                    }
                    out.push_str(&self.functor_decl(cid));
                }
                out.push_str(&format!("struct {} {{\n", ident(&def.name)));
                let mut names = Vec::new();
                for (field_idx, (field, field_ty)) in def.fields.iter().enumerate() {
                    let name = ident(field);
                    // A `()` field, such as an op value like `+`, holds nothing.
                    if self.devirt.dropped_field(*id, field_idx) {
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
                    if let Some(cid) = self.devirt.field_functor(*id, field_idx) {
                        out.push_str(&format!(
                            "    {} {name};\n",
                            crate::devirt::functor_name(cid)
                        ));
                        names.push(name);
                        continue;
                    }
                    self.need_ty(field_ty);
                    if self.lent_fields.is_lent(*id, field_idx) {
                        let cpp = self.ty(field_ty);
                        out.push_str(&format!("    const {cpp}* {name} = nullptr;\n"));
                        names.push(format!("*{name}"));
                        continue;
                    }
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
                        cpp = self.narrow_field_cpp(*id, field_idx, *w);
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
                if self.ties && Ty::Struct(*id).is_comparable(program) {
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
                    "enum class {} : {} {{ {} }};\n",
                    ident(&def.name),
                    tag_cpp(names.len()),
                    names.join(", ")
                ));
            }
            Ty::Enum(id) => {
                let def = &program.enums[*id];
                let name = ident(&def.name);
                let tags: Vec<String> = def.variants.iter().map(|(v, _)| ident(v)).collect();
                out.push_str(&format!(
                    "struct {name} {{\n    enum Tag : {} {{ {} }} tag = {};\n",
                    tag_cpp(tags.len()),
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
                if self.ties {
                    out.push_str(&format!(
                        "    auto cpl_tie() const {{ return std::tie({}); }}\n",
                        tie.join(", ")
                    ));
                }
                out.push_str("};\n");
            }
            _ => {}
        }
    }
}

/// The smallest unsigned type that holds an enum's `variants` tags: a grid
/// of a two-colour enum is a byte per cell, not four.
fn tag_cpp(variants: usize) -> &'static str {
    match variants {
        0..=256 => "uint8_t",
        257..=65536 => "uint16_t",
        _ => "uint32_t",
    }
}
