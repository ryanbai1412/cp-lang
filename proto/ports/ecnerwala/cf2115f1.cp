// Codeforces 2115 F1: Gellyfish and Lycoris Radiata.
// Maintain n sets under online prefix-insert of query id q, prefix-reverse,
// global delete of a query id, and point query for the smallest alive id.
// Store the sequence of positions in a reversible implicit treap. Each node
// holds `own` (tags at this node) and `lazy` (tags pending push to its
// subtree) as mergeable binary segment trees over Query. Reversal pushes
// lazily; queries walk root-to-node without pushing tags down, lazily pruning
// dead leaves in place so every deleted leaf is visited once.
use std.rand.Rng

struct SegNode(l: ?Int, r: ?Int)

struct TreapNode(l: ?Int, r: ?Int, pri: U64, own: ?Int, lazy: ?Int, sz: Int, flip: Bool)

let (n, q_cnt): (1..=300_000, 1..=300_000) = read()
domain Query = 1..=q_cnt
var seg: List<SegNode> = []
var alive: [Query]Bool = false

fn new_seg(l: ?Int, r: ?Int) -> Int:
    seg.push(SegNode(l, r))
    seg.len - 1

fn build_seg(q: Int, lo: Int, hi: Int) -> Int:
    if lo + 1 >= hi: return new_seg(none, none)
    let mid = (lo + hi) / 2
    if q < mid: new_seg(build_seg(q, lo, mid), none)
    else: new_seg(none, build_seg(q, mid, hi))

fn concat_seg(x: ?Int, y: ?Int, lo: Int, hi: Int) -> ?Int:
    let a = x ?? return y
    let b = y ?? return x
    let mid = (lo + hi) / 2
    let lc = concat_seg(seg[a].l, seg[b].l, lo, mid)
    let rc = concat_seg(seg[a].r, seg[b].r, mid, hi)
    new_seg(lc, rc)

fn query_seg(s: ?Int, lo: Int, hi: Int) -> (?Int, ?Query):
    let u = s ?? return (none, none)
    if lo + 1 == hi:
        let q = Query(lo + 1)
        return if alive[q]: (u, q) else: (none, none)
    let mid = (lo + hi) / 2
    let (nl, rl) = query_seg(seg[u].l, lo, mid)
    seg[u].l = nl
    if let v = rl: return (u, v)
    let (nr, rr) = query_seg(seg[u].r, mid, hi)
    seg[u].r = nr
    if let v = rr: (u, v) else: (none, none)

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
    let lz = tr[t].lazy ?? return
    if let lc = tr[t].l: tr[lc].lazy = concat_seg(tr[lc].lazy, lz, 0, q_cnt)
    if let rc = tr[t].r: tr[rc].lazy = concat_seg(tr[rc].lazy, lz, 0, q_cnt)
    tr[t].own = concat_seg(tr[t].own, lz, 0, q_cnt)
    tr[t].lazy = none

fn split_treap(u: ?Int, l: Int) -> (?Int, ?Int):
    let r = u ?? return (none, none)
    if l == 0: return (none, r)
    if l == tr[r].sz: return (r, none)
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
    let cur = u ?? return none
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

var root: ?Int = none
for _ in 0..n:
    tr.push(TreapNode(none, none, rng.next(), none, none, 1, false))
    root = merge_treap(root, tr.len - 1)

var last = 0
fn decode(x: Int, lim: Int) -> Int = (x + last - 1) % lim + 1

for q in Query:
    let (kind, b, c): (1..=3, Int, Int) = read()
    match kind:
        1 =>
            let r = decode(b, n)
            alive[q] = true
            let (ta, tb) = split_treap(root, r)
            if let a = ta:
                tr[a].lazy = concat_seg(tr[a].lazy, build_seg(q.to_idx, 0, q_cnt), 0, q_cnt)
            root = merge_treap(ta, tb)
        2 =>
            let r = decode(b, n)
            let (ta, tb) = split_treap(root, r)
            flip_treap(ta)
            root = merge_treap(ta, tb)
        3 => alive[Query(decode(b, q_cnt))] = false
    let p = decode(c, n) - 1
    last = if let v = query_treap(root, p): v.to_idx + 1 else: 0
    print(last)
