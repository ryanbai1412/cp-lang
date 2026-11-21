@!multitest
let (n, m): (Int, Int) = read()
domain Pos = 0..n
let ev: [0..m](t: Int, l: Int, r: Int) = read()

let sz = 4 * n + 4
var zero_lazy: [0..sz]Int = 0
var zero_full: [0..sz]Bool = false

fn zero_upd(node: Int, lo: Int, hi: Int, ql: Int, qr: Int, delta: Int):
    if qr <= lo or hi <= ql: return
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
    if zero_full[node] or lo >= limit: return none
    if lo + 1 == hi: return lo
    let mid = (lo + hi) / 2
    if limit > mid:
        if let res = find_prev_empty(node * 2 + 1, mid, hi, limit): return res
    find_prev_empty(node * 2, lo, mid, limit)

fn find_next_empty(node: Int, lo: Int, hi: Int, start: Int) -> ?Int:
    if zero_full[node] or hi <= start: return none
    if lo + 1 == hi: return lo
    let mid = (lo + hi) / 2
    if start < mid:
        if let res = find_next_empty(node * 2, lo, mid, start): return res
    find_next_empty(node * 2 + 1, mid, hi, start)

fn has_empty(node: Int, lo: Int, hi: Int, ql: Int, qr: Int) -> Bool:
    if zero_full[node] or qr <= lo or hi <= ql: return false
    if ql <= lo and hi <= qr: return true
    let mid = (lo + hi) / 2
    has_empty(node * 2, lo, mid, ql, qr) or has_empty(node * 2 + 1, mid, hi, ql, qr)

var one = SegTree<Pos, Int>(min)
var live: [Pos]Map<Int, Int> = Map()

fn sync_one(p: Pos):
    one.set(p, if let f = live[p].first: f.0 else: n + 1)

var min_l: [0..m]Int = 0
var l_ptr = 0
for r_ptr in 0..m:
    let (t, l0, ar) = ev[r_ptr]
    let al = l0 - 1
    while true:
        let ok = if t == 0:
            let prv = if al == 0: -1 else: find_prev_empty(1, 0, n, al) ?? -1
            let nxt = if ar == n: n else: find_next_empty(1, 0, n, ar) ?? n
            prv + 1 >= nxt or one.fold(Pos(prv + 1)..=Pos(nxt - 1)) > nxt
        else:
            has_empty(1, 0, n, al, ar)
        if ok:
            min_l[r_ptr] = l_ptr
            if t == 0:
                zero_upd(1, 0, n, al, ar, 1)
            else:
                let p = Pos(al)
                live[p][ar] += 1
                sync_one(p)
            break
        let (lt, ll0, lr) = ev[l_ptr]
        let ll = ll0 - 1
        if lt == 0:
            zero_upd(1, 0, n, ll, lr, -1)
        else:
            let p = Pos(ll)
            live[p][lr] -= 1
            if live[p][lr] == 0: live[p].remove(lr)
            sync_one(p)
        l_ptr += 1

let q_cnt: Int = read()
for _ in 0..q_cnt:
    let (ql, qr): (Int, Int) = read()
    print(if ql - 1 >= min_l[qr - 1]: "YES" else: "NO")
