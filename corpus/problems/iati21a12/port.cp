// Source: IATI 2021 A1 P2, https://dmoj.ca/problem/iati21a12
// Problem: a tree of n <= 2e5 nodes with weighted edges up to the parent; node v wants
//   want[v] units and gives give[v]. A unit moved up an edge of weight w gains w, and
//   serving demand at a node earns the unit's value. Print the largest total.
// Solution: go up from the leaves with one std MeldHeap of unclaimed supply per subtree,
//   absorbed small into large, whose lazy add_all adds the edge weight to all of them at
//   once. A node takes its most valuable units while they are worth at least 0; served
//   units re-enter at value 0 so a later node can re-route them. O(n log^2 n).
// original: corpus/submissions/5071843.cpp
// Bounds: n: 1..=2e5; p: 1..=n
use std.heap.{MeldHeap}

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

// supply[v]: the units still unclaimed in v's subtree, valued at v, most valuable first
var supply: [V]MeldHeap<Lot> = [V] _ => MeldHeap(by: l => -l.value)
var total = 0
for u in V.rev:
    for (v, w) in kids[u]:
        supply[v].add_all(Lot(w, 0))
        supply[u].absorb(mut supply[v])
    // serve u from the most valuable supply below, never at a loss
    var used = 0
    while used < want[u]:
        let lot = supply[u].pop() else: break
        if lot.value < 0:
            supply[u].push(lot)
            break
        let take = min(lot.count, want[u] - used)
        total += lot.value * take
        used += take
        if take < lot.count: supply[u].push(Lot(lot.value, lot.count - take))
    // served demand may be re-routed later, so it re-enters as supply at value 0
    supply[u].push(Lot(0, used))
    supply[u].push(Lot(0, give[u]))
print(total)
