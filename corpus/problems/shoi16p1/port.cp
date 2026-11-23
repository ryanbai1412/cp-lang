// Source: JLOI/SHOI 2016 (Scout Guards), https://dmoj.ca/problem/shoi16p1
//   (not public on DMOJ; statement from Luogu P3267)
// Problem: a tree on N <= 5e5 nodes; a guard at v costs W_v <= 1000 and watches every node
//   within distance D <= 20. Given M marked nodes, print the cheapest set of guards
//   watching all of them.
// Solution: tree DP on reach d in -D..D. need[u][d], d >= 0: u's subtree is settled and
//   a guard in it still covers d levels above u; d < 0: all is settled except its top -d
//   levels, left to a guard outside. need[u][d] is a suffix min over d. Reach D puts a guard
//   on u; 0 <= d < D lets one child bring reach d + 1 while the others need only -d; d < 0
//   asks d + 1 of every child; an unmarked u may leave itself open at d = 0. O(N D).
// original: corpus/submissions/5445691.cpp
let n: 1..=5e5 = read()
let k: 0..=20 = read()
domain V = 1..=n
let cost: [V]Int = read()
let m: 0..=n = read()
var marked: [V]Bool = false
for _ in 0..m:
    let u: V = read()
    marked[u] = true
var adj: [V]List<V> = []
for _ in 0..n - 1:
    let u: V = read()
    let v: V = read()
    adj[u].push(v)
    adj[v].push(u)

var kids: [V]List<V> = []
var seen: [V]Bool = false
var order: List<V> = []
order.push(V.first!)
seen[V.first!] = true
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for v in adj[u]:
        if not seen[v]:
            seen[v] = true
            kids[u].push(v)
            order.push(v)

// need[u, d + k]: cheapest way to settle u's subtree where
//   d >= 0: some guard in it reaches d levels above u,
//   d < 0:  the top -d levels of the subtree are covered from outside.
var need: [V, 0..=2 * k]Int = 0
for at in (0..n).rev:
    let u = order[at]
    for d in (-k..=k).rev:
        var best = 1e18
        if d < k: best = need[u, d + 1 + k]
        if d == k:
            var here = cost[u]
            for v in kids[u]: here += need[v, 0]
            best min= here
        elif d >= 0:
            // one child carries the guard, the rest only need its reach
            if kids[u].len > 0:
                var rest = 0
                var extra = 1e18
                for v in kids[u]:
                    rest += need[v, k - d]
                    extra min= need[v, d + 1 + k] - need[v, k - d]
                best min= rest + extra
        else:
            var below = 0
            for v in kids[u]: below += need[v, d + 1 + k]
            best min= below
        if d == 0 and not marked[u]:
            var below = 0
            for v in kids[u]: below += need[v, k]
            best min= below
        need[u, d + k] = best
print(need[V.first!, k])
