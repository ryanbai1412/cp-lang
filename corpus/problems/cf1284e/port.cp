// Source: Codeforces 1284E (New Year and Castle Construction),
//   https://codeforces.com/contest/1284/problem/E
// Problem: n <= 2500 distinct points, |x|, |y| <= 1e9, no three collinear. f(p) is the number of
//   4-subsets of the other points that form a quadrilateral strictly enclosing p. Print the sum
//   of f(p) over all p.
// Solution: count crossing segment pairs: a 4-subset in convex position has one crossing pair of
//   diagonals, a non-convex one none, and summed f(p) is (n - 4) per convex 4-subset minus one per
//   interior triangle point, which comes to (crossings - 2 choose(n, 4)) (n - 4) / 2. A line
//   rotating through every direction of a segment orders the points by projection; each segment
//   direction swaps its two adjacent ends, and the segment at positions (lo, lo + 1) then crosses
//   every segment from the lo points before it to the n - 2 - lo after. One angle sort of all
//   n^2 / 2 segments, O(n^2 log n).
use std.math.choose

// Every segment runs from its lexicographically smaller end, so all directions lie in one
// half-plane and `cross` alone orders them.
struct Dir(d: Pt)
impl Ord for Dir:
    fn lt(a, b) = cross(a.d, b.d) > 0
impl Eq for Dir:
    fn eq(a, b) = cross(a.d, b.d) == 0

let n: 5..=2500 = read()
let p = read<[0..n]Pt>().sorted(by: q => (q.x, q.y))
var segs = [(d: p[j] - p[i], a: i, b: j) for i in 0..n for j in i + 1..n]
segs.sort(by: s => Dir(s.d))
var at = (0..n).collect()  // the position of each point along the rotating line
var crossings = 0
for (_, a, b) in segs:
    crossings += at[a] * (n - 1 - at[b])
    at.swap_idx(a, b)
print((crossings - 2 * choose(n, 4)) * (n - 4) / 2)
