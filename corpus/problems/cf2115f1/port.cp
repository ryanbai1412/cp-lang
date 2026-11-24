// Source: Codeforces 2115F1 (Gellyfish and Lycoris Radiata, Easy Version),
//   https://codeforces.com/contest/2115/problem/F1
// limits: 3000 ms, 1024 MB
// Problem: n sets, all empty, and q ops (n, q <= 1e5), decoded online with the last answer. Op i
//   first inserts i into sets 1..r, reverses the order of sets 1..r, or deletes x from every set;
//   then print the smallest element of set p (0 if empty).
// Solution: keep the sets' order in an implicit treap with lazy reversal; each node carries the
//   ids tagged on it (`own`) and pending for its subtree (`lazy`) as shared mergeable binary
//   segment trees over the op ids, in an `Arena` whose node `empty` is the empty set. Insert tags
//   the split-off prefix; splits and merges push tags down, so every id at a node is smaller than
//   those at its ancestors. A query walks root to p without pushing and returns the deepest alive
//   id (leftmost alive leaf), pruning dead leaves in place so each is visited once. O(log) treap
//   work per op plus the segment-tree merges.
// Bounds: n, q_cnt: 1..=100_000; p: 1..=n
use std.marks.{Arena, Id}
use std.rand.Rng

struct SegNode(l: Id<SegNode>, r: Id<SegNode>)

let (n, q_cnt): (1..=100_000, 1..=100_000) = read()
domain Pos = 1..=n
domain Query = 1..=q_cnt

struct TreapNode:
    var l: ?Pos
    var r: ?Pos
    var pri: U64
    var own: Id<SegNode>
    var lazy: Id<SegNode>
    var sz: Int
    var flip: Bool

enum Op:
    @tag(1) Insert(r: Pos)
    @tag(2) Reverse(r: Pos)
    @tag(3) Delete(id: 1..=q_cnt)  // not `Query`: domain-only payloads misread tags
var seg = Arena<SegNode>.new()
var alive: [Query]Bool = false

fn new_seg(l: Id<SegNode>, r: Id<SegNode>) -> Id<SegNode> = seg.alloc(SegNode(l, r))
let empty = new_seg(Id(0), Id(0))

fn build_seg(q: Int, lo: Int, hi: Int) -> Id<SegNode>:
    if lo + 1 == hi: return new_seg(empty, empty)
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

fn singleton(q: Query) -> Id<SegNode> = build_seg(q.ord, 0, q_cnt)
fn cat(a: Id<SegNode>, b: Id<SegNode>) -> Id<SegNode> = concat_seg(a, b, 0, q_cnt)
fn first_alive(u: Id<SegNode>) -> (Id<SegNode>, ?Query) = query_seg(u, 0, q_cnt)

var rng = Rng.new(48)
var tr: [Pos]TreapNode = [Pos] _ => TreapNode(None, None, rng.next(), empty, empty, 1, false)

fn sz_of(u: ?Pos) -> Int = if let t = u: tr[t].sz else: 0

fn flip_treap(u: ?Pos):
    let t = u ?? return
    tr[t].flip = not tr[t].flip
    (tr[t].l, tr[t].r) = (tr[t].r, tr[t].l)

fn update_treap(t: Pos):
    tr[t].sz = sz_of(tr[t].l) + 1 + sz_of(tr[t].r)

fn push_flip(t: Pos):
    if not tr[t].flip: return
    flip_treap(tr[t].l)
    flip_treap(tr[t].r)
    tr[t].flip = false

fn push_down(t: Pos):
    push_flip(t)
    let lz = tr[t].lazy
    if lz == empty: return
    if let lc = tr[t].l: tr[lc].lazy = cat(tr[lc].lazy, lz)
    if let rc = tr[t].r: tr[rc].lazy = cat(tr[rc].lazy, lz)
    tr[t].own = cat(tr[t].own, lz)
    tr[t].lazy = empty

fn split_treap(u: ?Pos, l: Int) -> (?Pos, ?Pos):
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

fn merge_treap(a: ?Pos, b: ?Pos) -> ?Pos:
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

fn query_treap(u: ?Pos, l: Int) -> ?Query:
    let cur = u ?? return None
    let l_sz = sz_of(tr[cur].l)
    if l != l_sz:
        push_flip(cur)
        let res = if l > l_sz: query_treap(tr[cur].r, l - l_sz - 1) else: query_treap(tr[cur].l, l)
        if let v = res: return v
    else:
        let (no, ro) = first_alive(tr[cur].own)
        tr[cur].own = no
        if let v = ro: return v
    let (nl, rl) = first_alive(tr[cur].lazy)
    tr[cur].lazy = nl
    rl

var root: ?Pos = Pos.fold((r, p) => merge_treap(r, p), None)

var last = 0
fn decode<D>(x: D) -> D = D.nth((x.ord + last) % D.count)

for q in Query:
    let (op, p): (Op, Pos) = read()
    match op:
        Insert(r) =>
            alive[q] = true
            let (ta, tb) = split_treap(root, decode(r).ord + 1)
            if let a = ta: tr[a].lazy = cat(tr[a].lazy, singleton(q))
            root = merge_treap(ta, tb)
        Reverse(r) =>
            let (ta, tb) = split_treap(root, decode(r).ord + 1)
            flip_treap(ta)
            root = merge_treap(ta, tb)
        Delete(id) => alive[decode(Query(id))] = false
    last = if let v = query_treap(root, decode(p).ord): Int(v) else: 0
    print(last)
