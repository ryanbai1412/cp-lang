// Tuned: each level splits the caps into two plain lists with one pass instead of grouping them
// into a map by answer, and a single cap is paired with its single bottle without a query.
//
// Source: CCO 2009 Day 2 P1 (Bottle Caps), https://dmoj.ca/problem/cco09p4
// limits: 7000 ms, 256 MB
// Problem: interactive: n <= 1e4 caps and n bottles of distinct sizes, each cap matching one
//   bottle. Query "0 c b" answers 1 if cap c is too large for bottle b, 0 if it fits, -1 if
//   too small; report each match as "1 c b". At most 5e5 queries.
// Solution: quicksort the two sets against each other: compare every cap with a random
//   pivot bottle to split the caps three ways and find its cap, report the pair, partition
//   the other bottles by that cap, and recurse on the smaller and the larger halves. One query
//   per element per level: about 4 n ln n = 3.7e5 expected queries at n = 1e4.
// Bounds: n: 1..=10_000
use std.rand.{Rng}

let n: 1..=10_000 = read()
domain Cap = 1..=n
domain Bottle = 1..=n
fn fit(c: Cap, b: Bottle) -> -1..=1 = ask(0, c, b)
var rng = Rng.new(2009)
fn pair_up(caps: List<Cap>, bottles: List<Bottle>):
    if caps.is_empty: return
    if caps.len == 1:
        print(1, caps[0], bottles[0])
        return
    let pivot = rng.pick(bottles)!
    var smaller: List<Cap> = []
    var larger: List<Cap> = []
    var cap = caps[0]
    for c in caps:
        let f = fit(c, pivot)
        if f < 0: smaller.push(c)
        elif f > 0: larger.push(c)
        else: cap = c
    print(1, cap, pivot)
    var below: List<Bottle> = []
    var above: List<Bottle> = []
    for b in bottles if b != pivot:
        if fit(cap, b) > 0: below.push(b) else: above.push(b)
    pair_up(smaller, below)
    pair_up(larger, above)
pair_up(Cap.all, Bottle.all)
