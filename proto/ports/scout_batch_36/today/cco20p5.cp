// CCO '20 P5 — Interval Collection (today form for proto/cpc)
// original: corpus/submissions/4562195.cpp
let q: 1..=500_000 = read()
domain Q = 0..q

var ev_is_add: [Q]Bool = false
var ev_l: [Q]Int = 0
var ev_r: [Q]Int = 0
var max_coord = 1

for i in 0..q:
    let tok: [domain K]Char = read()
    let l: Int = read()
    let r: Int = read()
    ev_is_add[Q(i)] = tok[K(0)] == 'A'
    ev_l[Q(i)] = l
    ev_r[Q(i)] = r
    max_coord max= r + 2

let MAXC = max_coord
let STRIDE = MAXC + 5

// Segment tree over time [0, q - 1] storing active interval spans.
domain Seg = 0..4 * q + 5
var seg_l: [Seg]List<Int> = []
var seg_r: [Seg]List<Int> = []

fn add_span(o: Int, l: Int, r: Int, ql: Int, qr: Int, ivl: Int, ivr: Int):
    if ql <= l and r <= qr:
        seg_l[Seg(o)].push(ivl)
        seg_r[Seg(o)].push(ivr)
        return
    let mid = (l + r) / 2
    if ql <= mid: add_span(o * 2, l, mid, ql, qr, ivl, ivr)
    if mid < qr: add_span(o * 2 + 1, mid + 1, r, ql, qr, ivl, ivr)

// LIFO open stacks keyed by packed (l, r).
var open_head = HashMap<Int, Int>()
var st_prev: List<Int> = []
var st_time: List<Int> = []
var st_l: List<Int> = []
var st_r: List<Int> = []
var paired: [Q]Int = 0

for i in 0..q:
    let l = ev_l[Q(i)]
    let r = ev_r[Q(i)]
    let ky = l * STRIDE + r
    if ev_is_add[Q(i)]:
        let prev = open_head.get(ky) ?? -1
        let nid = st_time.len
        st_prev.push(prev)
        st_time.push(i)
        st_l.push(l)
        st_r.push(r)
        open_head[ky] = nid
    else:
        let nid = open_head[ky]
        let start = st_time[nid]
        open_head[ky] = st_prev[nid]
        paired[Q(i)] = start
        if start <= i - 1:
            add_span(1, 0, q - 1, start, i - 1, l, r)

// Remaining still-open intervals stay active through q - 1.
var is_closed_start: [Q]Bool = false
for i in 0..q:
    if not ev_is_add[Q(i)]:
        is_closed_start[Q(paired[Q(i)])] = true
for nid in 0..st_time.len:
    let s = st_time[nid]
    if not is_closed_start[Q(s)]:
        add_span(1, 0, q - 1, s, q - 1, st_l[nid], st_r[nid])

// Two mirrored rollback Fenwick trees (bit1 normal, bit2 reversed).
let INF = 1_000_000_000
var b1: List<Int> = []
var b2: List<Int> = []
for _ in 0..MAXC + 2:
    b1.push(0)
    b2.push(0)

var best1 = INF
var best2 = INF
var b1_hist: List<Int> = []
var b2_hist: List<Int> = []
var r1_hist: List<Int> = []
var r2_hist: List<Int> = []
var off1: List<Int> = [0]
var off2: List<Int> = [0]
var old1: List<Int> = []
var old2: List<Int> = []

fn lowbit(x: Int) -> Int: x & -x

fn ins1(l: Int, r: Int):
    b1_hist.push(best1)
    var hi = 0
    var i = l
    while i > 0:
        hi max= b1[i]
        i -= lowbit(i)
    if hi > 0: best1 min= r - hi
    r1_hist.push(r)
    var j = r
    while j <= MAXC:
        old1.push(b1[j])
        b1[j] max= l
        j += lowbit(j)
    off1.push(old1.len)

fn ins2(l: Int, r: Int):
    b2_hist.push(best2)
    var hi = 0
    var i = l
    while i > 0:
        hi max= b2[i]
        i -= lowbit(i)
    if hi > 0: best2 min= r - hi
    r2_hist.push(r)
    var j = r
    while j <= MAXC:
        old2.push(b2[j])
        b2[j] max= l
        j += lowbit(j)
    off2.push(old2.len)

fn rb1():
    let idx = r1_hist.len - 1
    var j = r1_hist[idx]
    r1_hist.pop()
    let lo = off1[idx]
    let hi = off1[idx + 1]
    off1.pop()
    for p in lo..hi:
        b1[j] = old1[p]
        j += lowbit(j)
    while old1.len > lo: old1.pop()
    best1 = b1_hist[b1_hist.len - 1]
    b1_hist.pop()

fn rb2():
    let idx = r2_hist.len - 1
    var j = r2_hist[idx]
    r2_hist.pop()
    let lo = off2[idx]
    let hi = off2[idx + 1]
    off2.pop()
    for p in lo..hi:
        b2[j] = old2[p]
        j += lowbit(j)
    while old2.len > lo: old2.pop()
    best2 = b2_hist[b2_hist.len - 1]
    b2_hist.pop()

var disjoint_ans: [Q]Int = INF

fn dfs(o: Int, l: Int, r: Int):
    let cnt = seg_l[Seg(o)].len
    for k in 0..cnt:
        let cl = seg_l[Seg(o)][k]
        let cr = seg_r[Seg(o)][k]
        ins1(cl, cr)
        ins2(MAXC - cr, MAXC - cl)
    if l == r:
        disjoint_ans[Q(l)] = min(best1, best2)
    else:
        let mid = (l + r) / 2
        dfs(o * 2, l, mid)
        dfs(o * 2 + 1, mid + 1, r)
    for _ in 0..cnt:
        rb1()
        rb2()

dfs(1, 0, q - 1)

// Three binary min-heaps with lazy deletion for overlapping pairs & singletons.
struct HItem:
    k1: Int
    k2: Int
    id: Int

fn h_less(a: HItem, b: HItem) -> Bool:
    if a.k1 != b.k1: return a.k1 < b.k1
    if a.k2 != b.k2: return a.k2 < b.k2
    a.id < b.id

var h1: List<HItem> = []
var h2: List<HItem> = []
var h3: List<HItem> = []

fn push1(it: HItem):
    h1.push(it)
    var i = h1.len - 1
    while i > 0:
        let p = (i - 1) / 2
        if h_less(h1[i], h1[p]):
            let tmp = h1[i]
            h1[i] = h1[p]
            h1[p] = tmp
            i = p
        else: break

fn pop1():
    let last = h1[h1.len - 1]
    h1.pop()
    if h1.len == 0: return
    h1[0] = last
    var i = 0
    while true:
        let l = i * 2 + 1
        let r = l + 1
        var sm = i
        if l < h1.len and h_less(h1[l], h1[sm]): sm = l
        if r < h1.len and h_less(h1[r], h1[sm]): sm = r
        if sm == i: break
        let tmp = h1[i]
        h1[i] = h1[sm]
        h1[sm] = tmp
        i = sm

fn push2(it: HItem):
    h2.push(it)
    var i = h2.len - 1
    while i > 0:
        let p = (i - 1) / 2
        if h_less(h2[i], h2[p]):
            let tmp = h2[i]
            h2[i] = h2[p]
            h2[p] = tmp
            i = p
        else: break

fn pop2():
    let last = h2[h2.len - 1]
    h2.pop()
    if h2.len == 0: return
    h2[0] = last
    var i = 0
    while true:
        let l = i * 2 + 1
        let r = l + 1
        var sm = i
        if l < h2.len and h_less(h2[l], h2[sm]): sm = l
        if r < h2.len and h_less(h2[r], h2[sm]): sm = r
        if sm == i: break
        let tmp = h2[i]
        h2[i] = h2[sm]
        h2[sm] = tmp
        i = sm

fn push3(it: HItem):
    h3.push(it)
    var i = h3.len - 1
    while i > 0:
        let p = (i - 1) / 2
        if h_less(h3[i], h3[p]):
            let tmp = h3[i]
            h3[i] = h3[p]
            h3[p] = tmp
            i = p
        else: break

fn pop3():
    let last = h3[h3.len - 1]
    h3.pop()
    if h3.len == 0: return
    h3[0] = last
    var i = 0
    while true:
        let l = i * 2 + 1
        let r = l + 1
        var sm = i
        if l < h3.len and h_less(h3[l], h3[sm]): sm = l
        if r < h3.len and h_less(h3[r], h3[sm]): sm = r
        if sm == i: break
        let tmp = h3[i]
        h3[i] = h3[sm]
        h3[sm] = tmp
        i = sm

var dead: [Q]Bool = false
for i in 0..q:
    let l = ev_l[Q(i)]
    let r = ev_r[Q(i)]
    if ev_is_add[Q(i)]:
        push1(HItem(r, -l, i))
        push2(HItem(-l, r, i))
        push3(HItem(r - l, 0, i))
    else:
        dead[Q(paired[Q(i)])] = true
    while dead[Q(h1[0].id)]: pop1()
    while dead[Q(h2[0].id)]: pop2()
    while dead[Q(h3[0].id)]: pop3()

    var best_inter = h3[0].k1
    var best_union = h3[0].k1
    let r1 = h1[0].k1
    let l1 = -h1[0].k2
    let l2 = -h2[0].k1
    let r2 = h2[0].k2
    if r1 >= l2:
        let ci = r1 - l2
        let cu = r2 - l1
        if ci < best_inter or (ci == best_inter and cu < best_union):
            best_inter = ci
            best_union = cu
    let dj = disjoint_ans[Q(i)]
    if dj < INF:
        if 0 < best_inter or (0 == best_inter and dj < best_union):
            best_inter = 0
            best_union = dj
    print(best_union)
