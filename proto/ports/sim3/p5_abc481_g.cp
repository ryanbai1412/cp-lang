// ABC481 G — Pancake Sorting Expectation
// NTT / Divide-and-Conquer Polynomial Multiplication via examples/local-lib/math/poly.cp + @brute(main)

use lib.math.poly.{Z, convolve}

let n: 1..=120_000 = read()
domain I = 0..n
let cakes: [I](a: 1..=998_244_352, b: 1..=998_244_352) = read()

// Precompute factorials, inverse factorials, and powers of 2
var fact: [0..=n]Z = Z(1)
var pow2: [0..=n]Z = Z(1)
for i in 1..=n:
    fact[i] = fact[i - 1] * Z(i)
    pow2[i] = pow2[i - 1] * Z(2)
var inv_fact: [0..=n]Z = Z(1)
inv_fact[n] = fact[n].inv
for i in (0..n).rev:
    inv_fact[i] = inv_fact[i + 1] * Z(i + 1)

fn binom_poly(d: Int) -> List<Z>:
    let fd = fact[d]
    [fd * inv_fact[i] * inv_fact[d - i] for i in 0..=d]

// Sweep sorted endpoints: (value, kind: 0 = A, 1 = B)
var events: List<(Int, Int)> = []
for i in I:
    events.push((cakes[i].a, 0))
    events.push((cakes[i].b, 1))
events.sort()

let m = 2 * n - 1
var ps: [0..m]Int = 0
var qs: [0..m]Int = 0
var ws: [0..m]Z = Z(0)

var cur_p = 0
var cur_q = 0
for t in 0..2 * n:
    if t > 0:
        let dv = events[t].0 - events[t - 1].0
        ps[t - 1] = cur_p
        qs[t - 1] = cur_q
        ws[t - 1] = if dv > 0: Z(dv) * pow2[n - cur_q] else: Z(0)
    if events[t].1 == 0:
        cur_q += 1
    else:
        cur_p += 1
        cur_q -= 1

fn dc(l: Int, r: Int) -> (List<Z>, Int, Int):
    if r - l == 1:
        return ([ws[l]], ps[l], qs[l])
    let mid = (l + r) / 2
    let (lp0, lp_min, lq_min) = dc(l, mid)
    let (rp0, rp_min, rq_min) = dc(mid, r)
    let pmin = lp_min
    let qmin = min(lq_min, rq_min)
    let lp = if lq_min > qmin: convolve(lp0, binom_poly(lq_min - qmin)) else: lp0
    let rp = if rq_min > qmin: convolve(rp0, binom_poly(rq_min - qmin)) else: rp0
    let shift = rp_min - pmin
    let out_len = max(lp.len, rp.len + shift)
    var out = List.filled(out_len, Z(0))
    for i in 0..lp.len:
        out[i] += lp[i]
    for i in 0..rp.len:
        out[i + shift] += rp[i]
    (out, pmin, qmin)

let (g0, p0, q0) = dc(0, m)
let g = if q0 > 0: convolve(g0, binom_poly(q0)) else: g0

var cur = pow2[n] * Z(events[0].0)
var ans: [I]Z = Z(0)
for k in 0..n:
    if k >= p0 and k - p0 < g.len:
        cur += g[k - p0]
    ans[I(k)] = cur

print(ans, sep: "\n")

@brute(main)
fn slow():
    var tot: [I]Z = Z(0)
    for mask in 0..(1 << n):
        var vals: List<Int> = []
        for i in I:
            if ((mask >> Int(i)) & 1) == 0:
                vals.push(cakes[i].a)
            else:
                vals.push(cakes[i].b)
        vals.sort()
        for i in I:
            tot[i] += Z(vals[Int(i)])
    print(tot, sep: "\n")
