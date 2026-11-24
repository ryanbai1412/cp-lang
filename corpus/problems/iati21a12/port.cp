// Source: IATI 2021 A1 P2, https://dmoj.ca/problem/iati21a12
// Problem: a tree of n <= 2e5 nodes with weighted edges up to the parent; node v wants
//   want[v] units and gives give[v]. A unit moved up an edge of weight w gains w, and
//   serving demand at a node earns the unit's value. Print the largest total.
// Solution: go up from the leaves with one heap of unclaimed supply per subtree, merged
//   small into large, and a per-heap lift that adds the edge weight to all of them at
//   once. A node takes its most valuable units while they are worth at least 0; served
//   units re-enter at value 0 so a later node can re-route them. O(n log^2 n).
// original: corpus/submissions/5071843.cpp
let n: 1..=2e5 = read()
domain V = 0..n
let want: [V]Int = read()
let give: [V]Int = read()
var kids: [V]List<(V, Int)> = []
for i in 1..n:
    let p: 1..=n = read()
    let w: Int = read()
    kids[V.nth(p - 1)].push((V.nth(i), w))

// pile[s]: (-(value - lift[s]), count) for the supply still unclaimed in a subtree
var pile: List<MinHeap<(Int, Int)>> = []
var lift: List<Int> = []
var slot: [V]Int = 0
for v in V:
    pile.push(MinHeap<(Int, Int)>())
    lift.push(0)
    slot[v] = v.ord

var total = 0
for u in V.rev:
    var mine = slot[u]
    for (v, w) in kids[u]:
        var other = slot[v]
        lift[other] += w
        if pile[mine].len < pile[other].len: swap(mut mine, mut other)
        while pile[other].len > 0:
            let (neg, cnt) = pile[other].pop()!
            pile[mine].push((neg - lift[other] + lift[mine], cnt))
    slot[u] = mine
    // serve u from the most valuable supply below, never at a loss
    var used = 0
    while used < want[u] and pile[mine].len > 0:
        let (neg, cnt) = pile[mine].pop()!
        let value = lift[mine] - neg
        if value < 0:
            pile[mine].push((neg, cnt))
            break
        let take = min(cnt, want[u] - used)
        total += value * take
        used += take
        if take < cnt: pile[mine].push((neg, cnt - take))
    // served demand may be re-routed later, so it re-enters as supply at value 0
    pile[mine].push((lift[mine], used))
    pile[mine].push((lift[mine], give[u]))
print(total)
