// n: 4096
// noi20p2's tree DP on a reserved pool: two `set`s per vertex, the second splitting the first's
// one-node chain, a prefix `sum` of each child, every child merged into its parent.
use std.segtree.{MergeTree, MergeTrees}

type Z = Mod<998_244_353>
let n: Int = read()
domain V = 0..n
let leaf: (Z, Z, Z, Z) -> Z = (va, vb, pa, pb) => va * (pb + vb) + vb * pa  // within: segtree.MergeTrees.merge
var s = Z(0)
for _ in 0..16:
    var p = MergeTrees<V, Z>()
    p.reserve(sets: 2 * n)
    var dp: List<MergeTree<V>> = []
    for i in 0..n:
        var t = MergeTree<V>()
        p.set(mut t, V(i * 7919 % n), Z(i + 1))
        p.set(mut t, V((i * 7919 + n / 2) % n), Z(i + 2))  // timed: segtree.MergeTrees.set
        dp.push(t)
    for i in 0..n:
        s += p.sum(dp[i], V.cuts[0]..V.cuts[i * 13 % n + 1])  // timed: segtree.MergeTrees.sum
    for i in (1..n).rev:
        let up = (i - 1) / 2
        dp[up] = p.merge(dp[up], dp[i], leaf, pre_b: Z(i))  // timed: segtree.MergeTrees.merge
    s += p.total(dp[0])
print(s)
