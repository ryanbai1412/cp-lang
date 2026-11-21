// CCC '02 S4 (Hard) — Bridge Crossing (today form for proto/cpc)
// original: corpus/submissions/4157744.cpp
let m: 1..=100_000 = read()
let n: 1..=100_000 = read()
domain P = 0..n
var names: List<Str> = []
var times: List<Int> = []
for _ in 0..n:
    let nm: Str = read()
    let tv: Int = read()
    names.push(nm)
    times.push(tv)

var sz = 1
while sz < n + 1: sz *= 2

let INF = 1_000_000_000_000_000_000
var arr_v: List<Int> = []
var dp_val: List<Int> = []
var dp_idx: List<Int> = []
var tot_val: List<Int> = []
var tot_idx: List<Int> = []
var lazy: List<Int> = []
for _ in 0..sz: arr_v.push(0)
for _ in 0..2 * sz:
    dp_val.push(INF)
    dp_idx.push(0)
    tot_val.push(INF)
    tot_idx.push(0)
    lazy.push(-1)

fn pull(o: Int):
    let lc = o * 2
    let rc = lc + 1
    if dp_val[lc] < dp_val[rc] or (dp_val[lc] == dp_val[rc] and dp_idx[lc] < dp_idx[rc]):
        dp_val[o] = dp_val[lc]
        dp_idx[o] = dp_idx[lc]
    else:
        dp_val[o] = dp_val[rc]
        dp_idx[o] = dp_idx[rc]
    if tot_val[lc] < tot_val[rc] or (tot_val[lc] == tot_val[rc] and tot_idx[lc] < tot_idx[rc]):
        tot_val[o] = tot_val[lc]
        tot_idx[o] = tot_idx[lc]
    else:
        tot_val[o] = tot_val[rc]
        tot_idx[o] = tot_idx[rc]

fn apply_tag(o: Int, x: Int):
    if x >= 0:
        lazy[o] = x
        tot_val[o] = dp_val[o] + x
        tot_idx[o] = dp_idx[o]

fn push(o: Int):
    let x = lazy[o]
    if x >= 0:
        apply_tag(o * 2, x)
        apply_tag(o * 2 + 1, x)
        lazy[o] = -1

fn build(o: Int, l: Int, r: Int):
    if l == r:
        dp_val[o] = 0
        dp_idx[o] = l
        tot_val[o] = 0
        tot_idx[o] = l
        return
    let mid = (l + r) / 2
    build(o * 2, l, mid)
    build(o * 2 + 1, mid + 1, r)
    pull(o)

fn st_set(o: Int, l: Int, r: Int, pos: Int, val: Int):
    if l == r:
        dp_val[o] = val
        dp_idx[o] = pos
        if lazy[o] >= 0:
            tot_val[o] = val + lazy[o]
        else:
            tot_val[o] = val
        tot_idx[o] = pos
        return
    push(o)
    let mid = (l + r) / 2
    if pos <= mid: st_set(o * 2, l, mid, pos, val)
    else: st_set(o * 2 + 1, mid + 1, r, pos, val)
    pull(o)

fn st_upd(o: Int, l: Int, r: Int, ql: Int, qr: Int, x: Int):
    if ql <= l and r <= qr:
        apply_tag(o, x)
        return
    push(o)
    let mid = (l + r) / 2
    if ql <= mid: st_upd(o * 2, l, mid, ql, qr, x)
    if mid < qr: st_upd(o * 2 + 1, mid + 1, r, ql, qr, x)
    pull(o)

var q_cost = INF
var q_at = 0

fn st_query(o: Int, l: Int, r: Int, ql: Int, qr: Int):
    if ql <= l and r <= qr:
        if tot_val[o] < q_cost or (tot_val[o] == q_cost and tot_idx[o] < q_at):
            q_cost = tot_val[o]
            q_at = tot_idx[o]
        return
    push(o)
    let mid = (l + r) / 2
    if ql <= mid: st_query(o * 2, l, mid, ql, qr)
    if mid < qr: st_query(o * 2 + 1, mid + 1, r, ql, qr)

build(1, 0, sz - 1)

var dp_c: List<Int> = []
var dp_nxt: List<Int> = []
for _ in 0..n + 1:
    dp_c.push(0)
    dp_nxt.push(n)

var stk_v: List<Int> = [INF]
var stk_i: List<Int> = [n]

var i = n - 1
while i >= 0:
    let t = times[i]
    while stk_v.last! < t:
        stk_v.pop()
        stk_i.pop()
    let hi = stk_i.last!
    st_upd(1, 0, sz - 1, i + 1, hi, t)
    stk_v.push(t)
    stk_i.push(i)
    let qr = min(n, i + m)
    q_cost = INF
    q_at = n
    st_query(1, 0, sz - 1, i + 1, qr)
    dp_c[i] = q_cost
    dp_nxt[i] = q_at
    st_set(1, 0, sz - 1, i, q_cost)
    i -= 1

print("Total Time:", dp_c[0])
var cur = 0
while cur < n:
    let end = dp_nxt[cur]
    var grp: List<Str> = [names[k] for k in cur..end]
    print(grp)
    cur = end
