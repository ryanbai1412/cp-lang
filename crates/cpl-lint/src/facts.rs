//! Prototype of the checker fact table (design/overview.md, M3): facts the
//! checker already computes, read by rules instead of re-derived from syntax.

use cpl_diag::Span;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Facts {
    /// The type a member belongs to, by the member name's file span:
    /// `Bitset` for the `first` of `b.first()`.
    owners: HashMap<(u32, u32), String>,
}

impl Facts {
    /// Checks `src` as the file `path` and keeps its facts.
    pub fn check(path: &str, src: &str) -> Facts {
        let mut db = cpl_check::Database::new();
        db.set_file(path, src);
        let Some(analysis) = db.analyze(path) else {
            return Facts::default();
        };
        let mut owners = HashMap::new();
        for item in &analysis.items {
            for entry in &item.check.hovers {
                if let ("field", Some(owner)) = (entry.keyword, &entry.owner) {
                    let span = entry.span.shift(item.offset);
                    owners.insert((span.start, span.end), owner.clone());
                }
            }
        }
        Facts { owners }
    }

    /// The type the member named at `span` belongs to.
    pub fn owner(&self, span: Span) -> Option<&str> {
        self.owners.get(&(span.start, span.end)).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_member_knows_the_type_it_belongs_to() {
        let src = "domain D = 0..10\nlet b = Bitset<D>()\nprint(b.first().is_some())\n";
        let facts = Facts::check("probe.cp", src);
        let at = src.find("first").unwrap() as u32;
        assert_eq!(facts.owner(Span::new(at, at + 5)), Some("Bitset"));
    }
}
