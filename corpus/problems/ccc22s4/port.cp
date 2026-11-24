// Source: CCC 2022 S4 (Good Triplets), https://dmoj.ca/problem/ccc22s4
// limits: 3000 ms, 1024 MB
// Problem: N <= 1e6 points at integer spots 0..C-1 (C <= 1e6, repeats allowed) of a circle of
//   circumference C. Count the triples of points whose triangle has the centre strictly inside.
// Solution: count the complement: a triple is bad iff it fits in a closed half circle. Charge it
//   to its first spot clockwise; the other two are there or within the next C / 2 spots, kept as
//   a sliding count. For even C, a triple with two points at one end of a diameter and one at the
//   other was charged at both ends, so subtract it once. O(N + C).
// original: corpus/submissions/4371347.cpp
// Bounds: n, c: 3..=1e6; points: 0..=c - 1

let (n, c): (3..=1e6, 3..=1e6) = read()
domain Spot = 0..c
let points: [n]Spot = read()
var at: [Spot]Int = 0
for p in points: at[p] += 1

// A triple is bad when it fits in a closed half circle. Charge it to its first spot going
// clockwise: the other two points are there too, or within the next c / 2 spots.
let half = c / 2
var bad = 0
var ahead = sum(at[Spot.wrap(k)] for k in 1..=half)
for p in Spot:
    bad += choose(at[p], 3) + choose(at[p], 2) * ahead + at[p] * choose(ahead, 2)
    ahead += at[Spot.wrap(p.ord + half + 1)] - at[Spot.wrap(p.ord + 1)]
// When c is even, a triple with two points on one end of a diameter and one on the other was
// charged at both ends.
if c % 2 == 0:
    for k in 0..half:
        let (p, q) = (Spot(k), Spot(k + half))
        bad -= choose(at[p], 2) * at[q] + at[p] * choose(at[q], 2)
print(choose(n, 3) - bad)
