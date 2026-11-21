// CCO '24 P5 — Parity-Alternating Subarray Partition DP (DMOJ: cco24p5)
// original: corpus/submissions/6435483.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Count partitions (mod 1_000_003) of array a[0..n) into valid contiguous blocks
// using two parity-indexed lazy segment trees over split boundaries I.cuts
// maintaining (min_bad, sum_dp_at_min) under range +1/-1 updates and point DP sets.
const MOD = 1_000_003
type Z = Mod<MOD>

let n: 1..=5e5 = read()
domain I = 0..n
type B = I.cuts
let a: [I](1..=n) = read()

// Note: Z (Mod<MOD>) is not Ord, so Node does not auto-derive Ord — legal in v2.
struct Node:
    mn: Int = 0
    ways: Z = 0

    fn merge(self, o: Node) -> Node:
        if self.mn < o.mn: self
        elif o.mn < self.mn: o
        else: Node(self.mn, self.ways + o.ways)

let add_bad = Action(
    id: 0,
    compose: (a, b) => a + b,
    apply: (node, d) => Node(node.mn + d, node.ways),
)

var dp: [B]Z = 0
dp[I.last.r] = 1

var nxt: [Bool, 1..=n]?I = none
var g_nxt: [1..=n]?I = none
var dup_end: [Bool, I]?B = none

var seg0 = LazySegTree<B, Node>(add_bad, [B] _ => Node(0, 0))
var seg1 = LazySegTree<B, Node>(add_bad, [B] _ => Node(0, 0))

fn step_parity(mut seg: LazySegTree<B, Node>, cb: Bool, i: I, x: 1..=n):
    let cur = seg.fold(i.r..=i.r)
    seg.set(i.r, Node(cur.mn, dp[i.r]))

    if let u = nxt[not cb, x]:
        seg.apply(u.r.., 1)

    let i_dup = (i.to_idx % 2 == 1) == cb
    if not i_dup:
        if let g = g_nxt[x]:
            seg.apply(g.r.., 1)
    else:
        let hi = if let d = nxt[cb, x]:
            seg.apply(d.r..=dup_end[cb, d]!, -1)
            d.l
        else:
            I.last.r
        dup_end[cb, i] = hi
        seg.apply(i.r..=hi, 1)

    let root = seg.fold(..)
    if root.mn == 0: dp[i.l] += root.ways

for i in I.rev:
    let x = a[i]
    step_parity(mut seg0, false, i, x)
    step_parity(mut seg1, true, i, x)
    nxt[i.to_idx % 2 == 1, x] = i
    g_nxt[x] = i

print(dp[I.first.l])
