//! Struct fn fields that only ever hold one capturing closure: the field is
//! that closure's own type (`cpl_clN`, a struct of its captures with a call
//! operator), so no `std::function` erases it.

use super::*;
use crate::uses::places::{each_built, each_written};

impl<'p> Devirt<'p> {
    pub(super) fn compute_functors(&mut self) {
        let mut held: BTreeMap<(usize, usize), Option<ClosureId>> = BTreeMap::new();
        let mut note = |key: (usize, usize), cid: Option<ClosureId>| {
            held.entry(key)
                .and_modify(|known| {
                    if *known != cid {
                        *known = None;
                    }
                })
                .or_insert(cid);
        };
        let mut scan = |block: &Block| {
            each_written(block, &mut |place| {
                if let (PlaceKind::Field(_, index), Ty::Struct(s)) =
                    (&place.kind, place_base_ty(place))
                {
                    note((*s, *index), None);
                }
            });
            each_built(block, &mut |s, fields| {
                for (index, field) in fields.iter().enumerate() {
                    if matches!(field.ty, Ty::Fn(..)) {
                        let cid = match field.kind {
                            ExprKind::Closure(cid) if self.functor_shaped(cid, s) => Some(cid),
                            _ => None,
                        };
                        note((s, index), cid);
                    }
                }
            });
        };
        scan(&self.program.setup);
        scan(&self.program.main);
        for f in &self.program.fns {
            scan(&f.body);
        }
        for closure in &self.program.closures {
            scan(&closure.body);
        }
        for (key, cid) in held {
            let Some(cid) = cid else { continue };
            if self.elidable.contains(&Home::Field(key.0, key.1)) {
                continue;
            }
            self.functor_fields.insert(key, cid);
            self.functors.insert(cid);
        }
    }

    /// A closure whose captures are all locals copied by value, each a
    /// scalar or a struct (not `holder`, which would then hold itself)
    /// holding no heap, so a member stores each exactly as the lambda would.
    fn functor_shaped(&self, cid: ClosureId, holder: usize) -> bool {
        let def = &self.program.closures[cid];
        !self.closure_effectively_captureless(cid)
            && def.captures.iter().all(|cap| {
                let ty = &def.locals[cap.inner].ty;
                !cap.is_global
                    && !cap.by_ref
                    && (ty.is_scalar() || matches!(ty, Ty::Struct(t) if *t != holder))
                    && !crate::lend::holds_heap(self.program, ty)
            })
    }

    /// The one closure struct `s`'s field `index` holds, stored as its own type.
    pub fn field_functor(&self, s: usize, index: usize) -> Option<ClosureId> {
        self.functor_fields.get(&(s, index)).copied()
    }

    /// Whether closure `cid` is built as a `cpl_clN` value rather than a lambda.
    pub fn functor(&self, cid: ClosureId) -> bool {
        self.functors.contains(&cid)
    }
}

pub(crate) fn functor_name(cid: ClosureId) -> String {
    format!("cpl_cl{cid}")
}
