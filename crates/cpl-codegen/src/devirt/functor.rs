//! Struct fn fields that only ever hold closures from a known, closed set:
//! the field is that closure's own type (`cpl_clN`, a struct of its captures
//! with a call operator) or, for several, a variant over their types that
//! calls through a switch on its tag (`cpl_clN_M`), so no `std::function`
//! erases it. One closure is the set of size one.

use super::*;
use crate::uses::places::{each_built, each_written};

impl<'p> Devirt<'p> {
    pub(super) fn compute_functors(&mut self) {
        // `None`: a closure the program can't list reaches the field.
        let mut held: BTreeMap<(usize, usize), Option<BTreeSet<ClosureId>>> = BTreeMap::new();
        let mut note = |key: (usize, usize), cid: Option<ClosureId>| {
            let known = held.entry(key).or_insert_with(|| Some(BTreeSet::new()));
            match (known.as_mut(), cid) {
                (Some(set), Some(cid)) => {
                    set.insert(cid);
                }
                _ => *known = None,
            }
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
                            // Built from a param or local that only ever holds one closure.
                            _ => match self.held.get(&Home::Field(s, index)) {
                                Some(&Held::One(cid)) if self.functor_shaped(cid, s) => Some(cid),
                                _ => None,
                            },
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
        for (key, set) in held {
            let Some(set) = set.filter(|set| !set.is_empty()) else {
                continue;
            };
            if self.elidable.contains(&Home::Field(key.0, key.1)) {
                continue;
            }
            self.functors.extend(&set);
            self.functor_fields.insert(key, set.into_iter().collect());
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
                !cap.is_global && (cap.by_ref || !matches!(ty, Ty::Struct(t) if *t == holder))
            })
    }

    /// The closures struct `s`'s field `index` holds, in id order: the field
    /// is the one's own type, or a variant over theirs.
    pub fn field_functor(&self, s: usize, index: usize) -> Option<&[ClosureId]> {
        self.functor_fields.get(&(s, index)).map(Vec::as_slice)
    }

    /// Whether closure `cid` is built as a `cpl_clN` value rather than a lambda.
    pub fn functor(&self, cid: ClosureId) -> bool {
        self.functors.contains(&cid)
    }
}

/// `cpl_cl3` for one closure's own type, `cpl_cl3_4` for the variant over two.
pub(crate) fn functor_name(cids: &[ClosureId]) -> String {
    let ids: Vec<String> = cids.iter().map(ToString::to_string).collect();
    format!("cpl_cl{}", ids.join("_"))
}
