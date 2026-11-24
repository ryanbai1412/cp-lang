// Source: CCO 2009 Day 2 P1 (Bottle Caps), https://dmoj.ca/problem/cco09p4
// Problem: interactive: n <= 1e4 caps and n bottles of distinct sizes, each cap matching one
//   bottle. Query "0 c b" answers 1 if cap c is too large for bottle b, 0 if it fits, -1 if
//   too small; report each match as "1 c b". At most 5e5 queries.
// Solution: quicksort the two sets against each other: compare every cap with a random
//   pivot bottle to split the caps three ways and find its cap, report the pair, partition
//   the other bottles by that cap, and recurse on the smaller and the larger halves. One query
//   per element per level: about 4 n ln n = 3.7e5 expected queries at n = 1e4.
// Bounds: n: 1..=10_000
let n: 1..=10_000 = read()
domain Cap = 1..=n
domain Bottle = 1..=n
fn fit(c: Cap, b: Bottle) -> -1..=1 = ask(0, c, b)
fn pair_up(caps: List<Cap>, bottles: List<Bottle>):
    if caps.len == 0: return
    let pivot = bottles[rand(0, bottles.len - 1)]
    let by_fit = caps.group(by: c => fit(c, pivot))
    let cap = by_fit[0][0]
    print(1, cap, pivot)
    let (below, above) = [b for b in bottles if b != pivot].partition(b => fit(cap, b) > 0)
    pair_up(by_fit.get(-1) ?? [], below)
    pair_up(by_fit.get(1) ?? [], above)
pair_up(Cap.all, Bottle.all)
