// Source: CCO 2019 P3 (Winter Driving), https://dmoj.ca/problem/cco19p3
// Problem: a tree of N <= 2e5 cities given by parents, city i holding a_i <= 1e4 people; every
//   city has at most 36 roads. Make each road one-way to maximise the ordered pairs of distinct
//   people (x, y) such that x can drive to y (same city always counts). Print the maximum.
// Solution: an optimum has a hub: each branch around it points all towards or all away from it.
//   chain[v]: with hub v, the pairs inside one branch or ending at the hub; it is rerooted from
//   parent to child in O(1). The cross pairs are s (everyone - a_hub - s), s the people in the
//   inward branches, so pick s nearest half: one branch if it outweighs the rest, else a
//   meet-in-the-middle over subset sums of the at most 36 branches. O(N + sum 2^(deg/2) deg).
// original: corpus/submissions/3618571.cpp
let n: 2..=2e5 = read()
domain V = 1..=n
domain Kid = 2..=n  // every city but the first has a parent
let a: [V]1..=1e4 = read()

fn random_parents() -> [Kid]V:
    [Kid] v => V(rand(1, v.idx + 1))

let parent: [Kid]V = read(gen: random_parents)
let everyone = a.sum()

// Every vertex after its parent, so the tree can be scanned up or down.
var kids: [V]List<V> = []
for v in Kid: kids[parent[v]].push(V(v.idx + 2))
var order = [V.first!]
var at = 0
while at < order.len:
    for c in kids[order[at]]: order.push(c)
    at += 1

var below: [V]Int = [V] v => a[v]  // people at or under v
for v in order.rev:
    for c in kids[v]: below[v] += below[c]

fn sums(xs: List<Int>) -> List<Int>:  // the total of every subset
    var out = [0]
    for x in xs:
        for more in [s + x for s in out]: out.push(more)
    out

// The largest total of some of the parts that is at most half of them all.
fn near_half(parts: List<Int>) -> Int:
    let whole = parts.sum()
    let big = parts.max()!
    if 2 * big >= whole: return whole - big  // one part outweighs the rest
    // Meet in the middle: all subsets of each half, then one sweep over both.
    let cut = parts.len / 2
    let ls = sums([parts[i] for i in 0..cut]).sorted()
    let rs = sums([parts[i] for i in cut..parts.len]).sorted()
    var best = 0
    var i = 0  // ls[i]: the largest that fits beside r
    for r in rs.rev:
        if 2 * r > whole: continue
        while i + 1 < ls.len and 2 * (ls[i + 1] + r) <= whole: i += 1
        best max= ls[i] + r
    best

// Pick a hub. Some of the branches around it drive towards the hub and the
// rest away from it. chain[v]: with hub v, the pairs that stay inside one
// branch or end at the hub, which is each person with everyone behind them.
// Moving the hub from v to its child c changes only those two terms.
var chain: [V]Int = 0
chain[V.first!] = sum(a[v] * (below[v] - a[v]) for v in V)
for v in order:
    for c in kids[v]: chain[c] = chain[v] + a[c] * (everyone - below[c]) - a[v] * below[c]

var best = 0
for v in V:
    var parts = [below[c] for c in kids[v]]
    if v != V.first!: parts.push(everyone - below[v])
    let s = near_half(parts)
    best max= chain[v] + s * (everyone - a[v] - s)
print(best + sum(x * (x - 1) for x in a))
