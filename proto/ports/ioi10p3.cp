// IOI 2010 Quality of Living: an r x c grid holds each of 1..r*c once. Print
// the smallest median of any h x w window (h and w odd).
use std.seq.{bsearch_first}

let (r, c, h, w): (1..=3000, 1..=3000, 1..=3000, 1..=3000) = read()
domain Row = 0..r
domain Col = 0..c
let rank: [Row, Col]Int = read()

// A window's median is at most x exactly when more than half of its cells are.
fn some_median_at_most(x: Int) -> Bool:
    let low = Prefix2<Row, Col>([Row, Col] (i, j) => Int(rank[i, j] <= x))
    any(2 * low.sum(i.l..i.r(h), j.l..j.r(w)) > h * w for i in Row.first..=Row(r - h) for j in Col.first..=Col(c - w))
print(bsearch_first(1..=r * c, some_median_at_most)!)
