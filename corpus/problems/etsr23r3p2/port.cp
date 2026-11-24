// Source: ETSR 2023 Round 3 P2, https://dmoj.ca/problem/etsr23r3p2
// Problem: a 100 x 100 sheet is folded n <= 10 times; fold i goes along the line from a
//   to b, and the part right of it is folded over to the left. For each of q <= 1e5
//   query points, print how many layers of paper lie under it.
// Solution: unfold backwards: a point on the left of the last fold was at itself or at
//   its mirror image before it, and nothing is on the right. Recurse over the folds, at
//   most 2^n paths per query.
// Bounds: n: 1..=10; q: 1..=1e5
// original: corpus/submissions/5529780.cpp
use std.geometry.{Vec}

let n: 1..=10 = read()
domain Fold = 1..=n
let folds: [Fold](a: Vec, b: Vec) = read()
let q: 1..=1e5 = read()
let holes: [q]Vec = read()

// The layers at p once the folds before `done` have been made.
fn layers(done: Cut<Fold>, p: Vec) -> Int:
    let last = done.l else: return if 0 <= p.x <= 100 and 0 <= p.y <= 100: 1 else: 0
    let (a, b) = folds[last]
    if (b - a).cross(p - a) < 0: return 0
    layers(done.prev!, p) + layers(done.prev!, reflect(p, a, b))
for p in holes: print(layers(Fold.end, p))
