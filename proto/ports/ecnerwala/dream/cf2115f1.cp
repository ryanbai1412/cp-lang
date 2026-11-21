// Codeforces 2115 F1: Gellyfish and Lycoris Radiata.
// Maintain n sets under online prefix-insert of query id q, prefix-reverse,
// global delete of a query id, and point query for the smallest alive id.
// Store the sequence of positions in a reversible implicit treap (Rope). Each
// node holds `own` (tags at this node) and `lazy` (tags pending push to its
// subtree) as mergeable binary segment trees over Query. Reversal pushes lazily;
// queries walk root-to-node without pushing tags down, lazily pruning dead
// leaves in place so every deleted leaf is visited once: O((n + q) log n log q).

use std.lazy.Rope
use std.segtree.MergeSeg

let (n, q_cnt): (1..=3e5, 1..=3e5) = read()
domain Pos = 1..=n
domain Query = 1..=q_cnt
var alive: [Query]Bool = false
var rope = Rope<Pos, MergeSeg<Query>>(
    merge: (a, b) => a.merge(b),
)

var last = 0
fn decode_pos(x: Int) -> Pos = Pos((x + last - 1) % n + 1)
fn decode_query(x: Int) -> Query = Query((x + last - 1) % q_cnt + 1)

for q in Query:
    let (kind, b, c): (1..=3, Int, Int) = read()
    match kind:
        1 =>
            alive[q] = true
            rope.apply(..=decode_pos(b), MergeSeg.singleton(q))
        2 => rope.reverse(..=decode_pos(b))
        3 => alive[decode_query(b)] = false
    let ans = rope.query_path(decode_pos(c), seg => seg.first_alive(id => alive[id]))
    last = if let v = ans: v.to_idx + 1 else: 0
    print(last)
