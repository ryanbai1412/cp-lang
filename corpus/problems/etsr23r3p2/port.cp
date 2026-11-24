// Source: ETSR 2023 Round 3 P2, https://dmoj.ca/problem/etsr23r3p2
// Problem: a 100 x 100 sheet is folded n <= 10 times; fold i goes along the line from a
//   to b, and the part right of it is folded over to the left. For each of q <= 1e5
//   query points, print how many layers of paper lie under it.
// Solution: unfold backwards: a point on the left of the last fold was at itself or at
//   its mirror image before it, and nothing is on the right. Recurse over the folds, at
//   most 2^n paths per query.
// original: corpus/submissions/5529780.cpp. No today/ form: every coordinate is a float (F15).
// Bounds: n: 1..=10; q: 1..=1e5
use std.geometry.{Vec, reflect}  // WISH F1, K17

let n: 1..=10 = read()
domain Fold = 1..=n
let folds: [Fold](a: Vec, b: Vec) = read()
let q: 1..=1e5 = read()
let holes: [q]Vec = read()

// Unfold backwards: paper at p after a fold was at p or at its mirror image
// before it, and nothing is left on the right of the fold line.
fn layers(done: Cut<Fold>, p: Vec) -> Int:                                        // the folds before `done` have been made
    let last = done.l else: return if 0 <= p.x <= 100 and 0 <= p.y <= 100: 1 else: 0  // WISH F5-like: the fold left of a boundary
    let (a, b) = folds[last]
    if (b - a).cross(p - a) < 0: return 0
    layers(done.prev!, p) + layers(done.prev!, reflect(p, a, b))  // WISH K17
for p in holes: print(layers(Fold.end, p))
