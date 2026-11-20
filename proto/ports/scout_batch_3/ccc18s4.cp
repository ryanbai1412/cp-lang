// CCC 2018 S4 Balanced Trees: a tree of weight w is a leaf (w = 1), or a
// root with k >= 2 subtrees that are all balanced trees of weight w / k,
// rounded down. Count the trees of weight n.
use std.math.{quotients}                                            // WISH G12: runs of equal w / k

let n: 1..=1e9 = read()

@memo
fn trees(w: Int) -> Int:
    if w == 1: return 1
    // many k share one quotient: count the run, solve the quotient once
    sum(ks.len * trees(q) for (q, ks) in quotients(w, 2..=w))
print(trees(n))
