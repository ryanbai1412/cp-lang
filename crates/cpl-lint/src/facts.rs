//! The checker fact table (design/overview.md, M3): what `cpl-check`
//! already knows about one file, read by rules instead of re-derived from
//! syntax. Built once from an `Analysis`, so `cpl lint` and the editor read
//! the same facts the checker reported from.

use cpl_check::{Analysis, Ty, TypedIndex};
use cpl_diag::Span;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Facts {
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
        let mut facts = Facts::default();
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
        facts
    }

    /// Checks `src` as the file `path`; `None` when the checker cannot
    /// analyze it.
    pub fn check(path: &str, src: &str) -> Option<Facts> {
        let mut db = cpl_check::Database::new();
        db.set_file(path, src);
        db.analyze(path).map(|a| Facts::from_analysis(&a))
    }

    /// The type of the place `span` names (`x`, `g[i][j]`, `self.best`): the
    /// widest typed span inside it that ends where it ends.
    pub fn place_type(&self, span: Span) -> Option<&Ty> {
        self.types
            .iter()
            .filter(|(s, _)| s.end == span.end && s.start >= span.start)
            .min_by_key(|(s, _)| s.start)
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
    use super::*;

    fn at(src: &str, text: &str) -> Span {
        let start = src.find(text).unwrap() as u32;
        Span::new(start, start + text.len() as u32)
    }

    #[test]
    fn a_member_knows_the_type_it_belongs_to() {
        let src = "domain D = 0..10\nlet b = Bitset<D>()\nprint(b.first().is_some())\n";
        let facts = Facts::check("probe.cp", src).unwrap();
        assert_eq!(facts.owner(at(src, "first")), Some("Bitset"));
    }

    #[test]
    fn a_place_knows_its_type() {
        let src = "var x: ?Int = Some(3)\nvar g: List<List<Int>> = [[1, 2], [3, 4]]\nx = x + g[0][1]\nprint(x ?? 0)\n";
        let facts = Facts::check("probe.cp", src).unwrap();
        let line = src.find("x = x").unwrap() as u32;
        let x = Span::new(line, line + 1);
        assert_eq!(facts.place_type(x), Some(&Ty::Optional(Box::new(Ty::Int))));
        assert_eq!(facts.place_type(at(src, "g[0][1]")), Some(&Ty::Int));
    }

    #[test]
    fn an_index_over_a_domain_is_proven_by_its_type() {
        let src = "domain V = 0..5\nvar dist: [V]Int = 0\nfor v in V:\n    print(dist[v])\n";
        let facts = Facts::check("probe.cp", src).unwrap();
        let proof = facts.typed_index(at(src, "dist[v]")).unwrap();
        assert!(proof.proof.contains("V"), "{}", proof.proof);
        let other = "let xs = [1, 2]\nlet i = 1\nprint(xs[i])\n";
        let facts = Facts::check("probe.cp", other).unwrap();
        assert!(facts.typed_index(at(other, "xs[i]")).is_none());
    }
}
