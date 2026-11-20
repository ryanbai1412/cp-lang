// IOI 2010 Quality of Living: an r x c grid holds each of 1..r*c once. Print
// the smallest median of any h x w window (h and w odd).
let r: 1..=3000 = read()
let c: 1..=3000 = read()
let h: 1..=r = read()
let w: 1..=c = read()
domain Row = 1..=r
domain Col = 1..=c
let rank: [Row, Col]1..=r * c = read()

// A window's median is at most x exactly when more than half of its cells are.
fn some_median_at_most(x: Int) -> Bool:
    let low = prefix([Row, Col] (i, j) => rank[i, j] <= x)          // WISH F2: prefix of a Bool array counts
    any(2 * low.sum(rows, cols) > h * w for rows in Row.windows(h) for cols in Col.windows(w))   // WISH F6
print((1..=r * c).first_where(some_median_at_most)!)                // WISH F10: search a plain range
