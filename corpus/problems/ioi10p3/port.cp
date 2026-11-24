// Source: IOI 2010 P3 (Quality of Living), https://dmoj.ca/problem/ioi10p3
// limits: 4500 ms, 256 MB
// Problem: an r x c grid (r, c <= 3000) holds each of 1..r*c once. Print the smallest median of
//   any h x w window (h <= r, w <= c, both odd). Input: r c h w, then the grid row by row.
// Solution: binary search the answer x: some window has median <= x iff more than half its
//   cells are <= x. Each check fills a 2D prefix sum of [rank <= x] and tests every h x w
//   window in O(1): O(rc log(rc)) total.
// original: corpus/problems/ioi10p3/orig.cpp
// Bounds: r, c, h, w: 1..=3000; rank: 1..=9e6

let (r, c, h, w): (1..=3000, 1..=3000, 1..=3000, 1..=3000) = read()
domain Row = 0..r
domain Col = 0..c
let rank: [Row, Col](1..=9e6) = read()

// A window's median is at most x exactly when more than half of its cells are.
fn some_median_at_most(x: Int) -> Bool:
    var low = Prefix<Row, Col, Int>.filling()
    for i in Row:
        for j in Col: low.set(i, j, Int(rank[i, j] <= x))
    any(2 * low.fold(rows, cols) > h * w for rows in Row.windows(h) for cols in Col.windows(w))
print((1..=r * c).first_true(some_median_at_most)!)
