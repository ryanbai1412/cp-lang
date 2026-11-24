// Source: Codeforces 2187C (Jerry and Tom), https://codeforces.com/contest/2187/problem/C
// limits: 3000 ms, 256 MB
// Problem: a DAG on 1..n with edges u -> u + 1 and m extra edges u -> v (no two crossing).
//   Jerry must move each turn, Tom may move or wait; Tom wins on meeting, Jerry on reaching
//   n. f(x, y) = Tom's minimum number of moves to force a win from (x, y), 0 if he can't.
//   Print the sum of f over all ordered pairs x != y. n <= 2e5 (sum), m <= n - 2, multitest.
// Solution: the best step from i is to nxt[i], the farthest vertex i reaches in one edge;
//   these pointers form a tree rooted at n, and a pair whose tree LCA is p contributes the
//   smaller of the two depths below p. Sum it by merging depth tables up the tree, the
//   shorter into the longer (each bucket keeps a count and a subtree-size sum, so one pass
//   over the shorter table adds every cross pair). O(n) by long-path merging.
// Bounds: n: 2..=200_000; m: 0..=200_000; edges.u, edges.v: 1..=n

struct Bucket(cnt: Int, sum: Int)

fn merge_by_depth(mut into: List<Bucket>, from: List<Bucket>) -> Int:
    let na = into.len
    let nb = from.len
    var gained = 0
    for l in 1..=nb:
        let a = into[na - l]
        let b = from[nb - l]
        gained += (a.cnt * b.sum + b.cnt * a.sum) * l
        into[na - l] = Bucket(a.cnt + b.cnt, a.sum + b.sum)
    gained

@!multitest
let (n, m): (2..=200_000, 0..=200_000) = read()
domain V = 1..=n
var nxt: [V]V = [V] i => i.next ?? i
let edges: [0..m](u: V, v: V) = read()
for (u, v) in edges: nxt[u] max= v

var sz: [V]Int = 0
var by_depth: [V]List<Bucket> = []
var ans = 0

for i in V:
    sz[i] += 1
    by_depth[i].push(Bucket(1, sz[i]))
    if i < V.last!:
        let p = nxt[i]
        assert p != i
        sz[p] += sz[i]
        if by_depth[p].len < by_depth[i].len:
            swap(mut by_depth[p], mut by_depth[i])
        ans += merge_by_depth(mut by_depth[p], by_depth[i])
        by_depth[i] = []

print(ans)
