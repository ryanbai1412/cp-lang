// OLY 2020 Practice 139, buildable under proto/cpc today. See ../oly20practice139.cp.
// original: corpus/submissions/6415690.cpp
const B = 400

fn ceil_div(a: Int, b: Int) -> Int = (a + b - 1) / b

let n: 1..=200_000 = read()
domain I = 0..n
var xs: [I]Int = 0
var ys_in: [I]Int = 0
for i in I:
    let x: 0..=1_000_000_000 = read()
    let y: 0..=1_000_000_000 = read()
    xs[i] = x
    ys_in[i] = y

var order: List<(Int, Int)> = []
for i in I: order.push((xs[i], ys_in[i]))
order.sort()

let (Y, yvals) = compress(ys_in)
let m = Y.K
let nb = (m + B - 1) / B
domain Bk = 0..nb

var b_add: [Bk]Int = 0
var b_ptr: [Bk]Int = 0
var b_v: [0..m]Int = 0
var h_at: [Bk]List<Int> = []
var h_m: [Bk]List<Int> = []
var h_b: [Bk]List<Int> = []

fn rebuild(bi: Bk):
    h_at[bi].clear()
    h_m[bi].clear()
    h_b[bi].clear()
    b_ptr[bi] = 0
    let ql = bi.ord * B
    var qr = ql + B
    if qr > m: qr = m
    for idx in ql..qr:
        let sl = yvals[Y.from_idx(idx)!]
        let ic = sl * b_v[idx]
        while h_at[bi].len > 0:
            let last = h_at[bi].len - 1
            let tx = h_at[bi][last]
            if sl * tx + ic > h_m[bi][last] * tx + h_b[bi][last]:
                h_at[bi].pop()
                h_m[bi].pop()
                h_b[bi].pop()
            else:
                break
        if h_at[bi].len == 0:
            h_at[bi].push(0)
        else:
            let last = h_at[bi].len - 1
            h_at[bi].push(ceil_div(h_b[bi][last] - ic, sl - h_m[bi][last]))
        h_m[bi].push(sl)
        h_b[bi].push(ic)

for bi in Bk: rebuild(bi)

fn query_all() -> Int:
    var best = 0
    for bi in Bk:
        let x = b_add[bi]
        while b_ptr[bi] + 1 < h_at[bi].len and x >= h_at[bi][b_ptr[bi] + 1]:
            b_ptr[bi] += 1
        let p = b_ptr[bi]
        let cur = h_m[bi][p] * x + h_b[bi][p]
        if cur > best: best = cur
    best

fn add_y(y: Int):
    var lo = 0
    var hi = m
    while lo < hi:
        let mid = (lo + hi) / 2
        if yvals[Y.from_idx(mid)!] <= y: lo = mid + 1
        else: hi = mid
    let pos = lo
    let bind = pos / B
    for i in 0..bind: b_add[Bk.from_idx(i)!] += 1
    let rem = pos % B
    if rem > 0:
        let base = bind * B
        for i in base..(base + rem): b_v[i] += 1
        rebuild(Bk.from_idx(bind)!)

var rem = n
var ans = 0
var i = 0
while i < n:
    let cur_x = order[i].0
    let cand = rem * cur_x + query_all()
    if cand > ans: ans = cand
    while i < n and order[i].0 == cur_x:
        add_y(order[i].1)
        i += 1
        rem -= 1
let tail = query_all()
if tail > ans: ans = tail
print(ans)
