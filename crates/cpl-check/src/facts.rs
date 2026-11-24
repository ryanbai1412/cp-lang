//! The fact table (design/overview.md, M3): what checking one file found
//! about each place, in file positions, for the tools that read it instead
//! of re-deriving it from syntax. Built once per revision on first use, so
//! `cpl lint`, the editor and `cpl fix` read the facts the checker reported from.

use crate::{Analysis, Revision, Ty, TypedIndex};
use cpl_diag::Span;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Facts {
    revision: Revision,
    /// The type of each name, member, call, index, unwrap and parenthesised
    /// expression, by file span.
    types: Vec<(Span, Ty)>,
    /// The type a member belongs to, by the member name's file span:
    /// `Bitset` for the `first` of `b.first()`.
    owners: HashMap<(u32, u32), String>,
    /// The indexes the checker proves by their type, `dist[v]` with `v: V`.
    typed_indexes: Vec<TypedIndex>,
}

impl Facts {
    pub fn from_analysis(analysis: &Analysis) -> Facts {
        let mut facts = Facts {
            revision: analysis.revision,
            ..Facts::default()
        };
        for item in &analysis.items {
            let shift = |s: Span| s.shift(item.offset);
            for entry in &item.check.hovers {
                facts.types.push((shift(entry.span), entry.ty.clone()));
                if let ("field", Some(owner)) = (entry.keyword, &entry.owner) {
                    let span = shift(entry.span);
                    facts.owners.insert((span.start, span.end), owner.clone());
                }
            }
            for (span, ty) in &item.check.shapes {
                facts.types.push((shift(*span), ty.clone()));
            }
            for t in &item.check.typed_indexes {
                facts.typed_indexes.push(TypedIndex {
                    access: shift(t.access),
                    seq: shift(t.seq),
                    index: shift(t.index),
                    proof: t.proof.clone(),
                });
            }
        }
        // By end, then start: `place_type` looks a place up by its end.
        facts.types.sort_by_key(|(s, _)| (s.end, s.start));
        facts
    }

    /// The session revision these facts were found at.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// The type of the place `span` names (`x`, `g[i][j]`, `self.best`): the
    /// widest typed span inside it that ends where it ends.
    pub fn place_type(&self, span: Span) -> Option<&Ty> {
        let at = self
            .types
            .partition_point(|(s, _)| (s.end, s.start) < (span.end, span.start));
        self.types
            .get(at)
            .filter(|(s, _)| s.end == span.end)
            .map(|(_, ty)| ty)
    }

    /// The type the member named at `span` belongs to.
    pub fn owner(&self, span: Span) -> Option<&str> {
        self.owners.get(&(span.start, span.end)).map(String::as_str)
    }

    /// The checker's proof that the index access at `access` is in bounds.
    pub fn typed_index(&self, access: Span) -> Option<&TypedIndex> {
        self.typed_indexes.iter().find(|t| t.access == access)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Database, Ty};
    use cpl_diag::Span;

    #[test]
    fn a_revision_builds_its_facts_once_in_file_positions() {
        let src = "let n = 3\nfn f(x: Int) -> Int:\n    x + n\nprint(f(n))\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        assert!(std::ptr::eq(analysis.facts(), analysis.facts()));
        let x = src.find("x + n").unwrap() as u32;
        assert_eq!(
            analysis.facts().place_type(Span::new(x, x + 1)),
            Some(&Ty::Int)
        );
    }
}
