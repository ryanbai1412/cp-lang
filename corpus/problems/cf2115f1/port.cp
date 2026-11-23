// Source: Codeforces 2115F1 (Gellyfish and Lycoris Radiata, Easy Version),
//   https://codeforces.com/contest/2115/problem/F1
// Problem: n sets, all empty, and q ops (n, q <= 3e5), decoded online with the last answer. Op i
//   first inserts i into sets 1..r, reverses the order of sets 1..r, or deletes x from every set;
//   then print the smallest element of set p (0 if empty).
// Solution: keep the sets' order in an implicit treap with lazy reversal; each node carries the
//   ids tagged on it (`own`) and pending for its subtree (`lazy`) as shared mergeable binary
//   segment trees over the op ids, in an `Arena` whose node `empty` is the empty set. Insert tags
//   the split-off prefix; splits and merges push tags down, so every id at a node is smaller than
//   those at its ancestors. A query walks root to p without pushing and returns the deepest alive
//   id (leftmost alive leaf), pruning dead leaves in place so each is visited once. O(log) treap
//   work per op plus the segment-tree merges.
use std.marks.{Arena, Id}
use std.rand.Rng

struct SegNode(l: Id<SegNode>, r: Id<SegNode>)

struct TreapNode:
    var l: ?Int
    var r: ?Int
    var pri: U64
    var own: Id<SegNode>
    var lazy: Id<SegNode>
    var sz: Int
    var flip: Bool

let (n, q_cnt): (1..=300_000, 1..=300_000) = read()
domain Pos = 1..=n
domain Query = 1..=q_cnt

enum Op:
    @tag(1) Insert(r: Pos)
    @tag(2) Reverse(r: Pos)
    @tag(3) Delete(id: 1..=q_cnt)  // not `Query`: domain-only payloads misread tags
var seg = Arena<SegNode>.new()
var alive: [Query]Bool = false

fn new_seg(l: Id<SegNode>, r: Id<SegNode>) -> Id<SegNode> = seg.alloc(SegNode(l, r))
let empty = new_seg(Id(0), Id(0))

fn build_seg(q: Int, lo: Int, hi: Int) -> Id<SegNode>:
    if lo + 1 >= hi: return new_seg(empty, empty)
    let mid = (lo + hi) / 2
    if q < mid: new_seg(build_seg(q, lo, mid), empty)
    else: new_seg(empty, build_seg(q, mid, hi))

fn concat_seg(a: Id<SegNode>, b: Id<SegNode>, lo: Int, hi: Int) -> Id<SegNode>:
    if a == empty: return b
    if b == empty: return a
    let mid = (lo + hi) / 2
    new_seg(concat_seg(seg[a].l, seg[b].l, lo, mid), concat_seg(seg[a].r, seg[b].r, mid, hi))

fn query_seg(u: Id<SegNode>, lo: Int, hi: Int) -> (Id<SegNode>, ?Query):
    if u == empty: return (empty, None)
    if lo + 1 == hi:
        let q = Query.nth(lo)
        return if alive[q]: (u, q) else: (empty, None)
    let mid = (lo + hi) / 2
    let (l, in_l) = query_seg(seg[u].l, lo, mid)
    seg[u] = SegNode(l, seg[u].r)
    if let q = in_l: return (u, q)
    let (r, in_r) = query_seg(seg[u].r, mid, hi)
    seg[u] = SegNode(l, r)
    if let q = in_r: (u, q) else: (empty, None)

var rng = Rng.new(48)
var tr: List<TreapNode> = []

fn sz_of(u: ?Int) -> Int = if let t = u: tr[t].sz else: 0

fn flip_treap(u: ?Int):
    let t = u ?? return
    tr[t].flip = not tr[t].flip
    (tr[t].l, tr[t].r) = (tr[t].r, tr[t].l)

fn update_treap(t: Int):
    tr[t].sz = sz_of(tr[t].l) + 1 + sz_of(tr[t].r)

fn push_flip(t: Int):
    if not tr[t].flip: return
    flip_treap(tr[t].l)
    flip_treap(tr[t].r)
    tr[t].flip = false

fn push_down(t: Int):
    push_flip(t)
    let lz = tr[t].lazy
    if lz == empty: return
    if let lc = tr[t].l: tr[lc].lazy = concat_seg(tr[lc].lazy, lz, 0, q_cnt)
    if let rc = tr[t].r: tr[rc].lazy = concat_seg(tr[rc].lazy, lz, 0, q_cnt)
    tr[t].own = concat_seg(tr[t].own, lz, 0, q_cnt)
    tr[t].lazy = empty

fn split_treap(u: ?Int, l: Int) -> (?Int, ?Int):
    let r = u ?? return (None, None)
    if l == 0: return (None, r)
    if l == tr[r].sz: return (r, None)
    push_down(r)
    let l_sz = sz_of(tr[r].l)
    if l <= l_sz:
        let (a, b_l) = split_treap(tr[r].l, l)
        tr[r].l = b_l
        update_treap(r)
        (a, r)
    else:
        let (a_r, b) = split_treap(tr[r].r, l - 1 - l_sz)
        tr[r].r = a_r
        update_treap(r)
        (r, b)

fn merge_treap(a: ?Int, b: ?Int) -> ?Int:
    let x = a ?? return b
    let y = b ?? return a
    if tr[x].pri < tr[y].pri:
        push_down(x)
        tr[x].r = merge_treap(tr[x].r, y)
        update_treap(x)
        x
    else:
        push_down(y)
        tr[y].l = merge_treap(x, tr[y].l)
        update_treap(y)
        y

fn query_treap(u: ?Int, l: Int) -> ?Query:
    let cur = u ?? return None
    let l_sz = sz_of(tr[cur].l)
    if l != l_sz:
        push_flip(cur)
        let res = if l > l_sz: query_treap(tr[cur].r, l - l_sz - 1) else: query_treap(tr[cur].l, l)
        if let v = res: return v
    else:
        let (no, ro) = query_seg(tr[cur].own, 0, q_cnt)
        tr[cur].own = no
        if let v = ro: return v
    let (nl, rl) = query_seg(tr[cur].lazy, 0, q_cnt)
    tr[cur].lazy = nl
    rl

var root: ?Int = None
for _ in Pos:
    tr.push(TreapNode(None, None, rng.next(), empty, empty, 1, false))
    root = merge_treap(root, tr.len - 1)

var last = 0
fn decode<D>(x: D) -> D = D.nth((x.ord + last) % D.count)

for q in Query:
    let (op, p): (Op, Pos) = read()
    match op:
        Insert(r) =>
            alive[q] = true
            let (ta, tb) = split_treap(root, decode(r).ord + 1)
            if let a = ta:
                tr[a].lazy = concat_seg(tr[a].lazy, build_seg(q.ord, 0, q_cnt), 0, q_cnt)
            root = merge_treap(ta, tb)
        Reverse(r) =>
            let (ta, tb) = split_treap(root, decode(r).ord + 1)
            flip_treap(ta)
            root = merge_treap(ta, tb)
        Delete(id) => alive[decode(Query(id))] = false
    last = if let v = query_treap(root, decode(p).ord): v.ord + 1 else: 0
    print(last)
