// Source: Codeforces 2219C, https://codeforces.com/contest/2219/problem/C
// Problem: multitest; a tree on n <= 2e5 nodes (sum n <= 2e5), each red or black, at least one
//   red. An operation picks a node and gives it the colour of a uniformly random neighbour.
//   Print the minimum expected number of operations to make all red (error 1e-6).
// Solution: each black node is operated on until it copies a red neighbour: deg / k expected
//   tries with k red neighbours. Root at a red node and DP bottom-up: dp0[u] when u's parent
//   is red first, dp1[u] when u turns red before it, from a child. Sort the children by
//   dp1 - dp0 and switch the cheapest k to dp1: u then waits deg / (k + 1) with the parent,
//   deg / k without. O(n log n).

@!multitest
let n: 1..=2e5 = read()
let s: Str = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

let rt = if let i = s.find('1'): V(i + 1) else: V(1)
let t = Rooted(g, root: rt)

var dp0: [V]F64 = 0.0
var dp1: [V]?F64 = None
for u in t.at.rev:
    var base = sum(dp0[c] for c in t.children(u))
    if s[u.ord] == '1':
        dp0[u] = base
        dp1[u] = base
        continue
    let ch = [d1 - dp0[c] for c in t.children(u) if let d1 = dp1[c]].sorted()
    let deg = F64(g[u].len)
    var b0 = base + deg
    var b1: ?F64 = None
    for (i, x) in ch.items:
        base += x
        b1 min= base + deg / F64(i + 1)
        b0 min= base + deg / F64(i + 2)
    dp0[u] = b0
    dp1[u] = b1

print("{dp0[rt]:.15f}")
