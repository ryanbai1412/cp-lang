// Source: CCO 2019 P3 (Winter Driving), https://dmoj.ca/problem/cco19p3
// limits: 600 ms, 1024 MB
// Problem: a tree of N <= 2e5 cities given by parents, city i holding a_i <= 1e4 people; every
//   city has at most 36 roads. Make each road one-way to maximise the ordered pairs of distinct
//   people (x, y) such that x can drive to y (same city always counts). Print the maximum.
// Solution: an optimum has a hub: each branch around it points all towards or all away from it.
//   chain[v]: with hub v, the pairs inside one branch or ending at the hub; a std `down` pass
//   reroots it from parent to child in O(1). The cross pairs are s (everyone - a_hub - s), s
//   the people in the inward branches, so pick s nearest half: one branch if it outweighs the
//   rest, else meet in the middle over std `subset_sums` of the two halves of the at most 36
//   branches. O(N + sum 2^(deg/2) deg).
// original: corpus/submissions/3618571.cpp
// Bounds: n: 2..=2e5; a: 1..=1e4
use std.dp.{subset_sums}

let n: 2..=2e5 = read()
domain V = 1..=n
domain Kid = 2..=n  // every city but the first has a parent
let a: [V]1..=1e4 = read()

fn random_parents() -> [Kid]V:
    [Kid] v => V(rand(1, v.idx + 1))

let parent: [Kid]V = read(gen: random_parents)
let t = Rooted.from_parents([V] v => Kid.at(Int(v)).map(k => parent[k]))
let everyone = a.sum()
let below = t.up(+, finish: (s, v) => s + a[v])  // people at or under v

// The largest total of some of the parts that is at most half of them all.
fn near_half(parts: List<Int>) -> Int:
    let (whole, big) = (parts.sum(), parts.max()!)
    if 2 * big >= whole: return whole - big  // one part outweighs the rest
    let cut = parts.len / 2
    let ls = subset_sums(parts[0..cut].collect())
    let rs = subset_sums(parts[cut..parts.len].collect())
    max(r + ls[ls.last_lt(whole / 2 - r + 1)!] for r in rs if 2 * r <= whole)!

// With hub v: the pairs that stay inside one branch or end at the hub, which is each person
// with everyone behind them. Moving the hub from p to its child v changes only those two terms.
let chain = t.down(sum(a[v] * (below[v] - a[v]) for v in V), (up, v) =>
    if let p = t.parent(v): up + a[v] * (everyone - below[v]) - a[p] * below[v] else: up)

fn parts(v: V) -> List<Int>:
    var out = [below[c] for c in t.children(v)]
    if let _ = t.parent(v): out.push(everyone - below[v])
    out

fn with_hub(v: V) -> Int:
    let s = near_half(parts(v))
    chain[v] + s * (everyone - a[v] - s)

print(max(with_hub(v) for v in V)! + sum(x * (x - 1) for x in a))
