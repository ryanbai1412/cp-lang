//! The checker's fact table, read by rules instead of re-derived from
//! syntax: `cpl_check::Facts`, from `Analysis::facts`.

pub use cpl_check::Facts;

#[cfg(test)]
mod tests {
    use cpl_check::Ty;
    use cpl_diag::Span;

    fn at(src: &str, text: &str) -> Span {
        let start = src.find(text).unwrap() as u32;
        Span::new(start, start + text.len() as u32)
    }

    #[test]
    fn a_member_knows_the_type_it_belongs_to() {
        let src = "domain D = 0..10\nlet b = Bitset<D>()\nprint(b.first().is_some())\n";
        let analysis = crate::analyze_alone(src).unwrap();
        let facts = analysis.facts();
        assert_eq!(facts.owner(at(src, "first")), Some("Bitset"));
    }

    #[test]
    fn a_place_knows_its_type() {
        let src = "var x: ?Int = Some(3)\nvar g: List<List<Int>> = [[1, 2], [3, 4]]\nx = x + g[0][1]\nprint(x ?? 0)\n";
        let analysis = crate::analyze_alone(src).unwrap();
        let facts = analysis.facts();
        let line = src.find("x = x").unwrap() as u32;
        let x = Span::new(line, line + 1);
        assert_eq!(facts.place_type(x), Some(&Ty::Optional(Box::new(Ty::Int))));
        assert_eq!(facts.place_type(at(src, "g[0][1]")), Some(&Ty::Int));
    }

    #[test]
    fn an_index_over_a_domain_is_proven_by_its_type() {
        let src = "domain V = 0..5\nvar dist: [V]Int = 0\nfor v in V:\n    print(dist[v])\n";
        let analysis = crate::analyze_alone(src).unwrap();
        let facts = analysis.facts();
        let proof = facts.typed_index(at(src, "dist[v]")).unwrap();
        assert!(proof.proof.contains("V"), "{}", proof.proof);
        let other = "let xs = [1, 2]\nlet i = 1\nprint(xs[i])\n";
        let analysis = crate::analyze_alone(other).unwrap();
        let facts = analysis.facts();
        assert!(facts.typed_index(at(other, "xs[i]")).is_none());
    }
}
