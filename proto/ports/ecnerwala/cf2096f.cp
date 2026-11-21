@! multitest
let n: Int = read()
let m: Int = read()
var ev_t: [0..m]Int = 0
var ev_l: [0..m]Int = 0
var ev_r: [0..m]Int = 0
for i in 0..m:
    let (t, l0, r0): (Int, Int, Int) = read()
    ev_t[i] = t
    ev_l[i] = l0 - 1
    ev_r[i] = r0

let sz = 4 * n + 4
var zero_lazy: [0..sz]Int = 0
var zero_full: [0..sz]Bool = false

fn zero_upd(node: Int, lo: Int, hi: Int, ql: Int, qr: Int, delta: Int):
    if qr <= lo or hi <= ql:
        return
    if ql <= lo and hi <= qr:
        zero_lazy[node] += delta
        zero_full[node] = zero_lazy[node] > 0 or (
            lo + 1 < hi and zero_full[node * 2] and zero_full[node * 2 + 1]
        )
        return
    let mid = (lo + hi) / 2
    zero_upd(node * 2, lo, mid, ql, qr, delta)
    zero_upd(node * 2 + 1, mid, hi, ql, qr, delta)
    zero_full[node] = zero_lazy[node] > 0 or (zero_full[node * 2] and zero_full[node * 2 + 1])

fn find_prev_empty(node: Int, lo: Int, hi: Int, limit: Int) -> ?Int:
    if zero_full[node] or lo >= limit:
        return none
    if lo + 1 == hi:
        return lo
    let mid = (lo + hi) / 2
    if limit > mid:
        if let res = find_prev_empty(node * 2 + 1, mid, hi, limit):
            return res
    find_prev_empty(node * 2, lo, mid, limit)

fn find_next_empty(node: Int, lo: Int, hi: Int, start: Int) -> ?Int:
    if zero_full[node] or hi <= start:
        return none
    if lo + 1 == hi:
        return lo
    let mid = (lo + hi) / 2
    if start < mid:
        if let res = find_next_empty(node * 2, lo, mid, start):
            return res
    find_next_empty(node * 2 + 1, mid, hi, start)

fn has_empty(node: Int, lo: Int, hi: Int, ql: Int, qr: Int) -> Bool:
    if zero_full[node] or qr <= lo or hi <= ql:
        return false
    if ql <= lo and hi <= qr:
        return true
    let mid = (lo + hi) / 2
    has_empty(node * 2, lo, mid, ql, qr) or has_empty(node * 2 + 1, mid, hi, ql, qr)

var one_min: [0..sz]Int = n + 1
var live: [0..n]Map<Int, Int> = Map()

fn one_set(node: Int, lo: Int, hi: Int, pos: Int, val: Int):
    if lo + 1 == hi:
        one_min[node] = val
        return
    let mid = (lo + hi) / 2
    if pos < mid:
        one_set(node * 2, lo, mid, pos, val)
    else:
        one_set(node * 2 + 1, mid, hi, pos, val)
    one_min[node] = min(one_min[node * 2], one_min[node * 2 + 1])

fn one_query(node: Int, lo: Int, hi: Int, ql: Int, qr: Int) -> Int:
    if qr <= lo or hi <= ql:
        return n + 1
    if ql <= lo and hi <= qr:
        return one_min[node]
    let mid = (lo + hi) / 2
    min(one_query(node * 2, lo, mid, ql, qr), one_query(node * 2 + 1, mid, hi, ql, qr))

var min_l: [0..m]Int = 0
var l_ptr = 0
for r_ptr in 0..m:
    let t = ev_t[r_ptr]
    let al = ev_l[r_ptr]
    let ar = ev_r[r_ptr]
    while true:
        var ok = false
        if t == 0:
            let prv = if al == 0: -1 else: find_prev_empty(1, 0, n, al) ?? -1
            let nxt = if ar == n: n else: find_next_empty(1, 0, n, ar) ?? n
            ok = one_query(1, 0, n, prv + 1, nxt) > nxt
        else:
            ok = has_empty(1, 0, n, al, ar)
        if ok:
            min_l[r_ptr] = l_ptr
            if t == 0:
                zero_upd(1, 0, n, al, ar, 1)
            else:
                let prev_cnt = if ar in live[al]: live[al][ar] else: 0
                live[al][ar] = prev_cnt + 1
                let cur_min = if let f = live[al].first: f.0 else: n + 1
                one_set(1, 0, n, al, cur_min)
            break
        else:
            let lt = ev_t[l_ptr]
            let ll = ev_l[l_ptr]
            let lr = ev_r[l_ptr]
            if lt == 0:
                zero_upd(1, 0, n, ll, lr, -1)
            else:
                let c = live[ll][lr] - 1
                if c == 0:
                    live[ll].remove(lr)
                else:
                    live[ll][lr] = c
                let cur_min = if let f = live[ll].first: f.0 else: n + 1
                one_set(1, 0, n, ll, cur_min)
            l_ptr += 1

let q_cnt: Int = read()
for _ in 0..q_cnt:
    let (ql, qr): (Int, Int) = read()
    print(if ql - 1 >= min_l[qr - 1]: "YES" else: "NO")
