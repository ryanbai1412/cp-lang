// ETSR 2023 R3 P2: a 100 x 100 sheet is folded n times. Fold i goes along the
// line from a to b, and the part to the right of it is folded over to the
// left. For each query point, print how many layers of paper lie under it.
// original: corpus/submissions/5529780.cpp. No today/ form: every coordinate is a float (F15).
use std.geometry.{cross, reflect}  // WISH F1, K17

let n: 1..=10 = read()
domain Fold = 1..=n
let folds: [Fold](a: Pt<F64>, b: Pt<F64>) = read()  // WISH F15, K17: points over a float
let q: 1..=1e5 = read()
let holes: [q]Pt<F64> = read()

// Unfold backwards: paper at p after a fold was at p or at its mirror image
// before it, and nothing is left on the right of the fold line.
fn layers(done: Cut<Fold>, p: Pt<F64>) -> Int:                                     // the folds before `done` have been made
    let last = done.l else: return if 0 <= p.x <= 100 and 0 <= p.y <= 100: 1 else: 0  // WISH F5-like: the fold left of a boundary
    let (a, b) = folds[last]
    if cross(b - a, p - a) < 0: return 0
    layers(done.prev, p) + layers(done.prev, reflect(p, a, b))  // WISH K17
for p in holes: print(layers(Fold.end, p))
