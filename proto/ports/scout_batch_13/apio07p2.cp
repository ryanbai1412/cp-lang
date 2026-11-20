// APIO 2007 Backup: n offices along a street. Lay k cables, each between two
// neighbouring offices, no office on two cables, using as little cable as
// possible.
// original: corpus/submissions/5532752.cpp
use std.convex.{Convex}                                             // WISH R8

let n: 2..=1e5 = read()
let k: 1..=n / 2 = read()
let at: Sorted<[n]0..=1e9> = read()                                 // WISH R5

// Choose k of the n - 1 gaps, no two next to each other. For a run of gaps
// keep costs[j], the cheapest j cables, once for each choice of which end
// gaps must stay empty. Every such list is convex in j.
type Run = [Bool, Bool]Convex               // [left end kept empty, right end kept empty]   WISH B11

fn one(gap: Int) -> Run = [Bool, Bool] (l, r) => if l or r: Convex([0]) else: Convex([0, gap])

// the two gaps that meet in the middle cannot both carry a cable
fn join(a: Run, b: Run) -> Run =
    [Bool, Bool] (l, r) => min(a[l, true].min_plus(b[false, r]), a[l, false].min_plus(b[true, r]))

let whole = fold_balanced(join, (one(q - p) for (p, q) in at.adjacent))     // WISH R7, S17
print(whole[false, false][k])
