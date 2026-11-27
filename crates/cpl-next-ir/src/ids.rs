//! Where a thing is, so that an edit stays local (handbook/design/next/incremental.md): an item
//! by its path, never by a byte offset or by the order modules were loaded in, and a place
//! inside it relative to its start.
use cpl_diag::Span;
use std::sync::Arc;

/// An item by its path: its module, its name, and which of that module's items with that
/// name it is (an impl's head, or the k-th block of top-level statements).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId {
    pub module: Arc<str>,
    pub name: Arc<str>,
    pub nth: u32,
}

impl ItemId {
    /// `span` of this item, counted from its start.
    pub fn at(&self, span: Span) -> SourceId {
        SourceId {
            item: self.clone(),
            span,
        }
    }
    /// A file as a whole: what lies before its first item, or in a file with none.
    pub fn file(module: Arc<str>) -> ItemId {
        ItemId {
            module,
            name: "".into(),
            nth: u32::MAX,
        }
    }
}

/// A place inside an item: a span counted from the item's start. Where the item itself is
/// in its file is the `spans` table's answer, the only thing a line added above it changes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId {
    pub item: ItemId,
    pub span: Span,
}
