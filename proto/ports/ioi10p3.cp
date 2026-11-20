// cp: 0.1 c217120+dirty
// IOI 2010 Quality of Living: an r x c grid holds each of 1..r*c once. Print
// the smallest median of any h x w window (h and w odd).
let r: 1..=3000 = read()
let c: 1..=3000 = read()
let h: 1..=r = read() where h % 2 == 1
let w: 1..=c = read() where w % 2 == 1
domain Row = 1..=r
domain Col = 1..=c
let rank: [Row, Col]1..=r * c = read()

// A window's median is at most x exactly when more than half of its cells are.
fn some_median_at_most(x: Int) -> Bool:
    let low = prefix([Row, Col] (i, j) => rank[i, j] <= x)
    any(2 * low.sum(rows, cols) > h * w for rows in Row.windows(h) for cols in Col.windows(w))
print((1..=r * c).first_where(some_median_at_most)!)
