// APIO '09 P2 (Siruseri Convention Centre): lexicographically smallest max
// independent set of closed intervals.
// Runnable on proto/cpc today.
// original: corpus/submissions/5536701.cpp
let n: 1..=2e5 = read()
domain I = 0..n
var segs: List<(Int, Int, Int)> = []
for i in 0..n:
    let l: Int = read()
    let r: Int = read()
    segs.push((l - 1, r, i))
segs.sort()

var pos: [0..n]Int = 0
for i in 0..n:
    let s = segs[i]
    pos[s.2] = i

// Sparse table over (r, idx)
const LOG = 18
var lg: [0..=n]Int = 0
for i in 2..=n: lg[i] = lg[i / 2] + 1

var st_r: [0..=LOG, I]Int = 0
var st_i: [0..=LOG, I]Int = 0
for i in I:
    let s = segs[i.idx]
    st_r[0, i] = s.1
    st_i[0, i] = i.idx
for k in 1..=LOG:
    let half = 1 << (k - 1)
    let span = 1 << k
    for i in 0..n:
        if i + span <= n:
            let lr = st_r[k - 1, I(i)]
            let li = st_i[k - 1, I(i)]
            let rr = st_r[k - 1, I(i + half)]
            let ri = st_i[k - 1, I(i + half)]
            if lr < rr or (lr == rr and li < ri):
                st_r[k, I(i)] = lr
                st_i[k, I(i)] = li
            else:
                st_r[k, I(i)] = rr
                st_i[k, I(i)] = ri

fn query_st(l: Int, r: Int) -> (Int, Int):
    if l >= r: return (2_000_000_000, -1)
    let k = lg[r - l]
    let p2 = r - (1 << k)
    let lr = st_r[k, I(l)]
    let li = st_i[k, I(l)]
    let rr = st_r[k, I(p2)]
    let ri = st_i[k, I(p2)]
    if lr < rr or (lr == rr and li < ri): (lr, li) else: (rr, ri)

fn first_after(x: Int) -> Int:
    let lo = (0..n).first_true(mid => segs[mid].0 >= x) ?? n
    lo

var lift: [0..=LOG, I]Int = -1
for i in 0..n:
    let p = query_st(first_after(segs[i].1), n)
    lift[0, I(i)] = p.1
for k in 1..=LOG:
    for i in 0..n:
        let mid = lift[k - 1, I(i)]
        if mid != -1: lift[k, I(i)] = lift[k - 1, I(mid)]

fn count_in(l: Int, r: Int, ql: Int, qr: Int) -> Int:
    if ql > qr: return -1_000_000_000
    let cql = max(first_after(ql), l)
    let p = query_st(cql, r)
    var u = p.1
    if u == -1 or segs[u].1 > qr: return 0
    var ans = 1
    for k in (0..=LOG).rev:
        let nu = lift[k, I(u)]
        if nu != -1 and nu < r and segs[nu].1 <= qr:
            u = nu
            ans += 1 << k
    let nxt = query_st(first_after(segs[u].1), r)
    if nxt.0 <= qr: ans += 1
    ans

var active = SegTree<I, Int>(-1, (a, b) => max(a, b))
var b_cr: [I]Int = 0
var b_ql: [I]Int = 0
var b_qr: [I]Int = 0
var b_v: [I]Int = 0

active.set(I(0), 0)
b_cr[I(0)] = n
b_ql[I(0)] = -1
b_qr[I(0)] = 1_000_000_000
b_v[I(0)] = count_in(0, n, -1, 1_000_000_000)

var ans: List<Int> = []
for qi in 0..n:
    let a = pos[qi]
    let cl = active.query(I(0), I(a))
    let cr = b_cr[I(cl)]
    let ql = b_ql[I(cl)]
    let qr = b_qr[I(cl)]
    let v = b_v[I(cl)]
    let s = segs[a]
    let vl = count_in(cl, a, ql, s.0)
    let vr = count_in(a + 1, cr, s.1, qr)
    if vl + 1 + vr == v:
        ans.push(qi + 1)
        active.set(I(cl), -1)
        if cl < a:
            active.set(I(cl), cl)
            b_cr[I(cl)] = a
            b_ql[I(cl)] = ql
            b_qr[I(cl)] = s.0
            b_v[I(cl)] = vl
        if a + 1 < cr:
            active.set(I(a + 1), a + 1)
            b_cr[I(a + 1)] = cr
            b_ql[I(a + 1)] = s.1
            b_qr[I(a + 1)] = qr
            b_v[I(a + 1)] = vr

print(ans.len)
print(ans)
