use std.rand.Rng

struct SegNode(l: Int, r: Int)

struct TreapNode(l: Int, r: Int, pri: U64, own: Int, lazy: Int, sz: Int, flip: Bool)

let (n, q_cnt): (Int, Int) = read()
var seg: List<SegNode> = [SegNode(0, 0)]
var alive: [0..q_cnt]Bool = false

fn new_seg(l: Int, r: Int) -> Int:
    seg.push(SegNode(l, r))
    seg.len - 1

fn build_seg(q: Int, lo: Int, hi: Int) -> Int:
    if lo + 1 >= hi: return new_seg(0, 0)
    let mid = (lo + hi) / 2
    if q < mid: new_seg(build_seg(q, lo, mid), 0)
    else: new_seg(0, build_seg(q, mid, hi))

fn concat_seg(x: Int, y: Int, lo: Int, hi: Int) -> Int:
    if x == 0: return y
    if y == 0: return x
    let mid = (lo + hi) / 2
    let lc = concat_seg(seg[x].l, seg[y].l, lo, mid)
    let rc = concat_seg(seg[x].r, seg[y].r, mid, hi)
    new_seg(lc, rc)

fn query_seg(s: Int, lo: Int, hi: Int) -> (Int, ?Int):
    if s == 0: return (0, none)
    if lo + 1 == hi: return if alive[lo]: (s, lo) else: (0, none)
    let mid = (lo + hi) / 2
    let (nl, rl) = query_seg(seg[s].l, lo, mid)
    seg[s].l = nl
    if let v = rl: return (s, v)
    let (nr, rr) = query_seg(seg[s].r, mid, hi)
    seg[s].r = nr
    if let v = rr: (s, v) else: (0, none)

var rng = Rng.new(48)
var tr: List<TreapNode> = [TreapNode(0, 0, U64.zero, 0, 0, 0, false)]

fn flip_treap(t: Int):
    tr[t].flip = not tr[t].flip
    (tr[t].l, tr[t].r) = (tr[t].r, tr[t].l)

fn update_treap(t: Int):
    tr[t].sz = tr[tr[t].l].sz + 1 + tr[tr[t].r].sz

fn push_flip(t: Int):
    if not tr[t].flip: return
    if tr[t].l != 0: flip_treap(tr[t].l)
    if tr[t].r != 0: flip_treap(tr[t].r)
    tr[t].flip = false

fn push_down(t: Int):
    push_flip(t)
    let lz = tr[t].lazy
    if lz == 0: return
    if tr[t].l != 0: tr[tr[t].l].lazy = concat_seg(tr[tr[t].l].lazy, lz, 0, q_cnt)
    if tr[t].r != 0: tr[tr[t].r].lazy = concat_seg(tr[tr[t].r].lazy, lz, 0, q_cnt)
    tr[t].own = concat_seg(tr[t].own, lz, 0, q_cnt)
    tr[t].lazy = 0

fn split_treap(r: Int, l: Int) -> (Int, Int):
    if l == 0: return (0, r)
    if l == tr[r].sz: return (r, 0)
    push_down(r)
    let l_sz = tr[tr[r].l].sz
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

fn merge_treap(a: Int, b: Int) -> Int:
    if a == 0: return b
    if b == 0: return a
    if tr[a].pri < tr[b].pri:
        push_down(a)
        tr[a].r = merge_treap(tr[a].r, b)
        update_treap(a)
        a
    else:
        push_down(b)
        tr[b].l = merge_treap(a, tr[b].l)
        update_treap(b)
        b

fn query_treap(cur: Int, l: Int) -> ?Int:
    let l_sz = tr[tr[cur].l].sz
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

var root = 0
for _ in 0..n:
    tr.push(TreapNode(0, 0, rng.next(), 0, 0, 1, false))
    root = merge_treap(root, tr.len - 1)

var prv_ans = 0
for q in 0..q_cnt:
    let (a, b, c): (Int, Int, Int) = read()
    if a == 1:
        let r = (b + prv_ans - 1) % n + 1
        alive[q] = true
        let (ta, tb) = split_treap(root, r)
        tr[ta].lazy = concat_seg(tr[ta].lazy, build_seg(q, 0, q_cnt), 0, q_cnt)
        root = merge_treap(ta, tb)
    elif a == 2:
        let r = (b + prv_ans - 1) % n + 1
        let (ta, tb) = split_treap(root, r)
        flip_treap(ta)
        root = merge_treap(ta, tb)
    else:
        alive[(b + prv_ans - 1) % q_cnt] = false
    let p = (c + prv_ans - 1) % n
    prv_ans = if let v = query_treap(root, p): v + 1 else: 0
    print(prv_ans)
