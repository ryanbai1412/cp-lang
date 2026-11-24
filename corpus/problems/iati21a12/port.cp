// Source: IATI 2021 A1 P2, https://dmoj.ca/problem/iati21a12
// Problem: a tree of n <= 2e5 nodes with weighted edges up to the parent; node v wants
//   want[v] units and gives give[v]. A unit moved up an edge of weight w gains w, and
//   serving demand at a node earns the unit's value. Print the largest total.
// Solution: go up from the leaves with one heap of unclaimed supply per subtree, all in one
//   std MeldHeaps pool so a child's heap melds in without copying, and whose lazy add_all adds
//   the edge weight to all of them at once. A node takes its most valuable units while they
//   are worth at least 0; served units re-enter at value 0 so a later node can re-route them.
//   O(n log n) amortized.
// original: corpus/submissions/5071843.cpp
// Bounds: n: 1..=2e5; p: 1..=n
use std.heap.{Meld, MeldHeaps}

let n: 1..=2e5 = read()
domain V = 0..n
let want: [V]Int = read()
let give: [V]Int = read()
var kids: [V]List<(V, Int)> = []
for i in 1..n:
    let (p, w): (1..=n, Int) = read()
    kids[V.nth(p - 1)].push((V.nth(i), w))

// `count` units worth `value` each; adding a lot lifts its value and keeps its count
struct Lot(value: Int, count: Int)
impl Add for Lot:
    fn add(x, y) = Lot(x.value + y.value, x.count)
impl Ord for Lot:
    fn lt(x, y) = x.value > y.value

// supply[v]: the units still unclaimed in v's subtree, valued at v, most valuable first; all of
// them live in one pool, so absorbing a child's heap melds it without copying
var pool = MeldHeaps<Lot>()
var supply: [V]Meld<Lot> = [V] _ => Meld()
var total = 0
for u in V.rev:
    for (v, w) in kids[u]:
        pool.add_all(supply[v], Lot(w, 0))
        supply[u] = pool.meld(supply[u], supply[v])
    // serve u from the most valuable supply below, never at a loss
    var used = 0
    while used < want[u]:
        let lot = pool.pop(mut supply[u]) else: break
        if lot.value < 0:
            pool.push(mut supply[u], lot)
            break
        let take = min(lot.count, want[u] - used)
        total += lot.value * take
        used += take
        if take < lot.count: pool.push(mut supply[u], Lot(lot.value, lot.count - take))
    // served demand may be re-routed later, so it re-enters as supply at value 0
    pool.push(mut supply[u], Lot(0, used))
    pool.push(mut supply[u], Lot(0, give[u]))
print(total)
