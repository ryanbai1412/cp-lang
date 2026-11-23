// IOI 2010 Quality of Living: an r x c grid holds each of 1..r*c once. Print
// the smallest median of any h x w window (h and w odd).

let (r, c, h, w): (1..=3000, 1..=3000, 1..=3000, 1..=3000) = read()
domain Row = 0..r
domain Col = 0..c
let rank: [Row, Col]Int = read()

// A window's median is at most x exactly when more than half of its cells are.
fn some_median_at_most(x: Int) -> Bool:
    let low = prefix([Row, Col] (i, j) => Int(rank[i, j] <= x))
    any(2 * low.sum(rows, cols) > h * w for rows in Row.windows(h) for cols in Col.windows(w))
print((1..=r * c).first_true(some_median_at_most)!)
