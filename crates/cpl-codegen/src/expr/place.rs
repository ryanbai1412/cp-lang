//! Places as C++: an lvalue, its index steps, and whether it is plain.

use super::*;

impl<'p> Emitter<'p> {
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
            PlaceKind::Global(id) => global_ident(self.program, *id),
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
                if let (Ty::Array { dims, .. }, PlaceKind::Local(_) | PlaceKind::Global(_)) =
                    (&base.ty, &base.kind)
                {
                    let joined = crate::flat::joined(dims, indices)
                        .filter(|_| self.grid_ck(&base.kind, indices).is_empty());
                    if let Some(groups) = joined {
                        let at: Vec<String> = groups.iter().map(|g| self.expr(g.0).text).collect();
                        let spans: Vec<usize> = groups.iter().map(|g| g.1).collect();
                        return crate::flat::cell(&b, &at, &spans);
                    }
                }
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
                let untrapped = self.target.checks == crate::Checks::None;
                match &base.ty {
                    Ty::Array { dims, .. } if dims.len() > 1 => {
                        format!(
                            "{b}{}({})",
                            self.grid_ck(&base.kind, indices),
                            at.join(", ")
                        )
                    }
                    Ty::Array { .. } | Ty::Str | Ty::List(_) | Ty::MutView(..) if untrapped => {
                        format!("{b}[{}]", at[0])
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
                    Ty::List(_) | Ty::MutView(..)
                        if matches!(indices[0].kind, ExprKind::Cast(Cast::Raw, _)) =>
                    {
                        format!("{b}[{}]", at[0])
                    }
                    Ty::List(_) | Ty::MutView(..) if self.inside_place(&base.kind, indices) => {
                        format!("{b}[{}]", at[0])
                    }
                    Ty::List(_) | Ty::MutView(..) => {
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
            plain &= !touch.io && touch.writes.is_empty() && touch.binds.is_empty();
        });
        plain
    }

    /// An element of a `{...}` list, which C++ refuses to narrow: a
    /// `Char` or position that C++ computed as a wider type is cast back.
    pub(super) fn braced(&mut self, e: &Expr) -> String {
        if e.ty == Ty::Unit {
            return self.stored(e);
        }
        let code = self.expr(e);
        let typed = matches!(
            e.kind,
            ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Field(..)
                | ExprKind::Char(_)
                | ExprKind::Read(_)
                | ExprKind::Parse(..)
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
}
