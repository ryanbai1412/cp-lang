// Source: AtCoder ABC481 G, https://atcoder.jp/contests/abc481/tasks/abc481_g
// Problem: n <= 120000 pancakes, pancake i has size a_i or b_i (a_i < b_i < 998244353) with
//   probability 1/2 each. For each k print 2^n E[k-th smallest size] mod 998244353.
// Solution: write the k-th smallest as v_min plus each gap dv between consecutive sorted
//   endpoints for which it exceeds the gap's low end. If p pancakes lie wholly below a gap
//   and q straddle it, 2^(n-q) C(q, j) choices put exactly p + j values below, so
//   ans[k] - ans[k-1] = [x^k] sum_t dv_t 2^(n-q_t) x^(p_t) (1 + x)^(q_t). Divide and conquer
//   over the gaps, factoring out x^p_min (1 + x)^q_min, with NTT products: O(n log^2 n).

use std.ntt.{convolve}

type Z = Mod<998_244_353>

enum End: Low | High
struct Event(val: Int, end: End)
struct Gap(p: Int, q: Int, w: Z)
struct Segment(poly: List<Z>, p_min: Int, q_min: Int)

fn add_shifted(a: List<Z>, b: List<Z>, shift: Int) -> List<Z>:
    var out = List.filled(max(a.len, b.len + shift), Z(0))
    for i in 0..a.len: out[i] += a[i]
    for i in 0..b.len: out[i + shift] += b[i]
    out

let n: 1..=120_000 = read()
domain I = 0..n
let cakes: [I](a: 1..=998_244_352, b: 1..=998_244_352) = read()

let comb = Comb<Z>(n)

fn binom_poly(d: Int) -> List<Z> = [comb.choose(d, i) for i in 0..=d]

var events: List<Event> = []
for i in I:
    events.push(Event(cakes[i].a, End.Low))
    events.push(Event(cakes[i].b, End.High))
events.sort(by: e => (e.val, e.end))

var gaps: List<Gap> = []
var cur_p = 0
var cur_q = 0
for (prev, cur) in events.pairs:
    match prev.end:
        End.Low => cur_q += 1
        End.High =>
            cur_p += 1
            cur_q -= 1
    let dv = cur.val - prev.val
    let w = if dv > 0: Z(dv) * (Z(2) ** (n - cur_q)) else: Z(0)
    gaps.push(Gap(cur_p, cur_q, w))

fn merge_gaps(l: Int, r: Int) -> Segment:
    if r - l == 1:
        return Segment([gaps[l].w], gaps[l].p, gaps[l].q)
    let mid = (l + r) / 2
    let left = merge_gaps(l, mid)
    let right = merge_gaps(mid, r)
    let q_min = min(left.q_min, right.q_min)
    let lp = if left.q_min > q_min: convolve(
        left.poly,
        binom_poly(left.q_min - q_min),
    ) else: left.poly
    let rp = if right.q_min > q_min: convolve(
        right.poly,
        binom_poly(right.q_min - q_min),
    ) else: right.poly
    Segment(add_shifted(lp, rp, right.p_min - left.p_min), left.p_min, q_min)

let top = merge_gaps(0, gaps.len)
let raw_diff = if top.q_min > 0: convolve(top.poly, binom_poly(top.q_min)) else: top.poly
let diff = add_shifted([], raw_diff, top.p_min)

let base = (Z(2) ** n) * Z(events[0].val)
let delta: [I]Z = [I] i => (if i == I.first: base else: Z(0)) + (diff.get(i.ord) ?? Z(0))
let ans = delta.scan((a, b) => a + b)

print(ans, sep: "\n")

@brute(main)
fn slow():
    var tot: [I]Z = Z(0)
    for mask in 0..(1 << n):
        let vals = [if ((mask >> i.ord) & 1) == 0: cakes[i].a else: cakes[i].b for i in I].sorted()
        for i in I:
            tot[i] += Z(vals[i.ord])
    print(tot, sep: "\n")
