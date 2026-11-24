//! A call whose result keeps a type-parameter argument lends that argument
//! (R05.5.3): `filter(xs, p)` is a `Filtered<S, T>` whose `src: S` is `xs`.

use super::*;

impl Checker<'_> {
    /// Calls `each` with every argument a call's result holds: each one
    /// passed for a parameter of type `S` where a struct field of the result
    /// type is `S`, transitively through generic arguments. A method's
    /// receiver is `base`.
    pub(crate) fn held_args<'e>(
        &mut self,
        call: &Expr,
        callee: &'e Expr,
        args: &'e [ast::Arg],
        each: &mut dyn FnMut(&mut Self, &'e Expr, bool),
    ) {
        if !self.is_held_value(call) || args.iter().any(|a| a.name.is_some()) {
            return;
        }
        let callee = match &unparen(callee).kind {
            ExprKind::Generic { base, .. } => unparen(base),
            _ => unparen(callee),
        };
        let (def, receiver) = match &callee.kind {
            ExprKind::Name(n) => (self.global(n), None),
            ExprKind::Field { base, name } if !self.has_own_method(base, &name.text) => {
                (self.chosen_method(name), Some(&**base))
            }
            _ => return,
        };
        let Some(Def::Fn {
            ty: Ty::Fn { params, ret },
            lends: false,
            ..
        }) = def.as_deref()
        else {
            return;
        };
        let skip = usize::from(receiver.is_some());
        for (i, param) in params.iter().enumerate() {
            let arg = match i.checked_sub(skip) {
                None => receiver.map(|r| (r, false)),
                Some(j) => args.get(j).map(|a| (&a.value, a.mutable)),
            };
            let Some((arg, is_mut)) = arg else { break };
            if let Ty::Param(g) = &param.ty {
                if self.holds(ret, g, 0) {
                    each(self, arg, is_mut);
                }
            }
        }
    }

    /// A struct's own method of the name (`SortedBag.take()`), whose `self`
    /// is the struct, not a type parameter it could hold.
    fn has_own_method(&mut self, base: &Expr, name: &str) -> bool {
        match self.infer_quietly(base) {
            Ty::Named { name: ty, .. } => self.struct_method(&ty, name).is_some(),
            _ => false,
        }
    }

    /// What passing `arg` to a call whose result keeps it holds: a heap or
    /// `mut` place, or what a view `arg` itself holds (`map(filter(xs, p), f)`).
    pub(crate) fn arg_hold(
        &mut self,
        arg: &Expr,
        is_mut: bool,
        kind: &'static str,
        out: &mut Vec<(&'static str, Place, bool, bool)>,
    ) {
        if !self.place_hold(arg, is_mut, kind, out) {
            out.append(&mut self.extra_holds(arg));
        }
    }

    /// Holds `arg`'s whole place if it is `mut` or heap-typed; false when
    /// `arg` is no place.
    pub(crate) fn place_hold(
        &mut self,
        arg: &Expr,
        is_mut: bool,
        kind: &'static str,
        out: &mut Vec<(&'static str, Place, bool, bool)>,
    ) -> bool {
        let Some(mut place) = self.place(arg) else {
            return false;
        };
        let arg_ty = self.infer_quietly(arg);
        if is_mut || self.is_heap_type(&arg_ty) {
            place.steps.clear();
            out.push((kind, place, !is_mut, is_mut));
        }
        true
    }

    /// Whether the `@get` getter `name` keeps its receiver: it yields, or its
    /// result holds the receiver's type parameter (`xs.items`).
    pub(crate) fn getter_holds_receiver(&mut self, e: &Expr, name: &ast::Name) -> bool {
        let def = self.chosen_method(name);
        let Some(Def::Fn {
            ty: Ty::Fn { params, ret },
            lends: false,
            getter: true,
            ..
        }) = def.as_deref()
        else {
            return false;
        };
        if matches!(&**ret, Ty::Named { name, .. } if name == "Gen") {
            return true;
        }
        if !self.is_held_value(e) {
            return false;
        }
        match params.first().map(|p| &p.ty) {
            Some(Ty::Param(g)) => self.holds(ret, g, 0),
            _ => false,
        }
    }

    /// Whether `e` may keep what it was made from: not a plain value or an
    /// owned container (an array's `map` is a new array, `xs.filter` a view).
    fn is_held_value(&mut self, e: &Expr) -> bool {
        let ty = match self.infer_quietly(e) {
            Ty::Optional(inner) => *inner,
            ty => ty,
        };
        let owned = matches!(&ty, Ty::Str | Ty::Array { .. } | Ty::Tuple(_))
            || matches!(&ty, Ty::Named { name, .. } if matches!(name.as_str(), "List" | "Map" | "Set"));
        !owned && !ty.is_plain_value()
    }

    /// Whether a struct field somewhere in `ty` keeps a `g`, through generic
    /// arguments, options and tuples. A result that is `g` itself (depth 0)
    /// is the argument moved out, not held.
    fn holds(&mut self, ty: &Ty, g: &str, depth: usize) -> bool {
        match ty {
            Ty::Param(p) => depth > 0 && p == g,
            Ty::Optional(inner) => self.holds(inner, g, depth),
            Ty::Tuple(ts) => depth > 0 && ts.iter().any(|t| self.holds(t, g, depth)),
            Ty::Named { name, args } if depth < 7 => {
                let def = self.type_def(name);
                let Some(Def::Struct {
                    generics, fields, ..
                }) = def.as_deref()
                else {
                    return false;
                };
                generics.iter().zip(args).any(|(h, arg)| {
                    self.holds(arg, g, depth + 1)
                        && fields.iter().any(|f| self.holds(&f.ty, h, depth + 1))
                })
            }
            _ => false,
        }
    }
}
