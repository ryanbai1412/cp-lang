// Codeforces 2115 F1: Gellyfish and Lycoris Radiata.
// Maintain n sets under online prefix-insert of query id q, prefix-reverse,
// global delete of a query id, and point query for the smallest alive id.
// Store the sequence of positions in a reversible implicit treap (Rope). Each
// node holds `own` (tags at this node) and `lazy` (tags pending push to its
// subtree) as mergeable binary segment trees over Query, sharing nodes in one
// MergeSegs pool. Reversal pushes lazily; queries walk root-to-node without
// pushing tags down, lazily pruning dead leaves in place so every deleted leaf
// is visited once: O((n + q) log n log q).

use std.lazy.{Rope}
use std.segtree.{MergeSeg, MergeSegs}

let (n, q_cnt): (1..=3e5, 1..=3e5) = read()
domain Pos = 1..=n
domain Query = 1..=q_cnt
var alive: [Query]Bool = false
var segs = MergeSegs<Query>()
var rope = Rope<Pos, MergeSeg<Query>>(
    fill: MergeSeg<Query>(),
    merge: (a, b) => segs.merge(a, b),
)

enum Op:
    @tag(1) Insert(r: Pos)
    @tag(2) Reverse(r: Pos)
    @tag(3) Delete(id: Query)

var last = 0
fn decode<D>(x: D) -> D = D.nth((x.ord + last) % D.count)

for q in Query:
    let (op, p): (Op, Pos) = read()
    match op:
        Insert(r) =>
            alive[q] = true
            rope.apply(..=decode(r), segs.singleton(q))
        Reverse(r) => rope.reverse(..=decode(r))
        Delete(id) => alive[decode(id)] = false
    let ans = rope.query_path(decode(p), (mut seg) => segs.first_alive(mut seg, id => alive[id]))
    last = if let v = ans: Int(v) else: 0
    print(last)
