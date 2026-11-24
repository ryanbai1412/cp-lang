// Source: COCI 2013/2014 Contest 1 (Ratar), https://dmoj.ca/problem/coci13c1p3
// Problem: an n x n grid (n <= 50) of values in [-1000, 1000]. Count the pairs of rectangles
//   that have exactly one corner point in common (touching diagonally) and equal sums.
// Solution: such a pair meets at one grid point, in opposite quadrants. For every grid point,
//   list the sums of all rectangles with a corner there in each quadrant (2-D prefix sums),
//   and count equal pairs between top-left and bottom-right, and between top-right and
//   bottom-left, with a count map: O(n^2) points times O(n^2) rectangles, O(n^4).
// Bounds: n: 1..=50; a: -1000..=1000
let n: 1..=50 = read()
domain I = 1..=n
let a: [I, I]-1000..=1000 = read()
let pre = prefix(a)

// pairs (x, y) with x == y
fn matches(xs: List<Int>, ys: List<Int>) -> Int:
    let seen = counts(xs)
    sum(seen[y] for y in ys)

var total = 0
for r in I.cuts:
    for c in I.cuts:
        // sums of the rectangles with one corner at grid point (r, c) and the
        // opposite corner cell in the given quadrant
        fn corner(rows: Range<I>, cols: Range<I>) -> List<Int>:
            [pre.fold(span(r, x), span(c, y)) for x in rows for y in cols]
        total += matches(corner(..r, ..c), corner(r.., c..)) + matches(
            corner(..r, c..),
            corner(r.., ..c),
        )
print(total)
