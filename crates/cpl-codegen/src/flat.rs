//! A grid read at the positions of a product domain, `d[p, q]` with
//! `p, q: (City, Corner)`, takes each position apart into one index per
//! dimension (`p.ord / n`, `p.ord % n`) only for the grid to put them back
//! together: `(o / n) * n + o % n == o`. The cell is the ordinal itself.
use cpl_ir::{BinOp, Cast, Dim, Expr, ExprKind};

fn bare(e: &Expr) -> &Expr {
    match &e.kind {
        ExprKind::Cast(Cast::Trust, inner) => bare(inner),
        _ => e,
    }
}

/// The indices of a grid with `dims`, each position taken apart put back
/// together: `(index, how many dimensions it spans)`. `None` when no two
/// indices join.
pub(crate) fn joined<'e>(dims: &[Dim], indices: &'e [Expr]) -> Option<Vec<(&'e Expr, usize)>> {
    if indices.len() != dims.len() {
        return None;
    }
    let mut groups: Vec<(&Expr, usize)> = Vec::with_capacity(indices.len());
    for (k, index) in indices.iter().enumerate() {
        if let (Some(&(prev, span)), ExprKind::Binary(BinOp::Rem, y, len)) =
            (groups.last(), &bare(index).kind)
        {
            let ExprKind::Binary(BinOp::Div, x, n) = &bare(prev).kind else {
                groups.push((index, 1));
                continue;
            };
            let own_len =
                matches!((&dims[k], &len.kind), (Dim::Pos(d), ExprKind::DomainLen(l)) if d == l);
            if own_len && x.kind == y.kind && n.kind == len.kind {
                if let Some(last) = groups.last_mut() {
                    *last = (x, span + 1);
                    continue;
                }
            }
        }
        groups.push((index, 1));
    }
    (groups.len() < indices.len()).then_some(groups)
}

/// The cell of grid `b` at the joined indices `at`, spanning `spans` dimensions each.
pub(crate) fn cell(b: &str, at: &[String], spans: &[usize]) -> String {
    let mut flat = at[0].clone();
    let mut dim = spans[0];
    for (code, &span) in at.iter().zip(spans).skip(1) {
        let size: Vec<String> = (dim..dim + span).map(|k| format!("{b}.n[{k}]")).collect();
        flat = format!("({flat}) * {} + {code}", size.join(" * "));
        dim += span;
    }
    format!("{b}.d[(size_t)({flat})]")
}
