//! What a place is rooted in.

use crate::{GlobalId, Place, PlaceKind, Slot};

impl Place {
    /// The local this place is in, when it is in one.
    pub fn root_local(&self) -> Option<Slot> {
        match &self.kind {
            PlaceKind::Local(slot) => Some(*slot),
            PlaceKind::Global(_) => None,
            PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                base.root_local()
            }
        }
    }

    /// The global this place is in, when it is in one.
    pub fn root_global(&self) -> Option<GlobalId> {
        match &self.kind {
            PlaceKind::Global(id) => Some(*id),
            PlaceKind::Local(_) => None,
            PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                base.root_global()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Place, PlaceKind, Ty};
    use cpl_diag::Span;

    fn place(kind: PlaceKind) -> Place {
        Place {
            kind,
            ty: Ty::Int,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn a_place_is_rooted_in_the_local_or_global_under_its_projections() {
        let local = place(PlaceKind::Some(Box::new(place(PlaceKind::Field(
            Box::new(place(PlaceKind::Local(3))),
            0,
        )))));
        assert_eq!((local.root_local(), local.root_global()), (Some(3), None));
        let global = place(PlaceKind::Index(
            Box::new(place(PlaceKind::Global(1))),
            vec![],
        ));
        assert_eq!((global.root_local(), global.root_global()), (None, Some(1)));
    }
}
