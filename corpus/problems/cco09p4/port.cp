// Source: CCO 2009 Day 2 P1 (Bottle Caps), https://dmoj.ca/problem/cco09p4
// Problem: interactive: n <= 1e4 caps and n bottles of distinct sizes, each cap matching one
//   bottle. Query "0 c b" answers 1 if cap c is too large for bottle b, 0 if it fits, -1 if
//   too small; report each match as "1 c b". At most 5e5 queries.
// Solution: quicksort the two sets against each other: compare every cap with a random
//   pivot bottle to split the caps and find its cap, report the pair, split the bottles by
//   that cap, and recurse on the smaller and the larger halves. Expected O(n log n) queries.
// As it has to be written today: the three-way split is three lists filled by hand.
//   Builds; 60 conversations with cco09p4_judge.py accepted for this file and 5034519.cpp.
let n: 1..=1000 = read()
domain Cap = 1..=n
domain Bottle = 1..=n
fn fit(c: Cap, b: Bottle) -> Int:
    let r: -1..=1 = ask(0, c, b)
    r
fn pair_up(caps: List<Cap>, bottles: List<Bottle>):
    if caps.len == 0: return
    let pivot = bottles[rand(0, bottles.len - 1)]
    var small: List<Cap> = []
    var big: List<Cap> = []
    var fits: ?Cap = None
    for c in caps:
        let r = fit(c, pivot)
        if r < 0: small.push(c)
        elif r > 0: big.push(c)
        else: fits = c
    let cap = fits!
    print(1, cap, pivot)
    let below = [b for b in bottles if b != pivot and fit(cap, b) > 0]
    let above = [b for b in bottles if b != pivot and fit(cap, b) < 0]
    pair_up(small, below)
    pair_up(big, above)
pair_up(Cap.all, Bottle.all)
