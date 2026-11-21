// ABC481 G — Pancake Sorting Expectation: sum over all `2^n` choices of the
// `k`-th smallest chosen pancake value (for every `k in 0..n`), where pancake `i`
// takes `a_i` or `b_i` (`a_i <= b_i`).
//
// Write each sorted value as `v_min + sum_{gaps} dv * [v > gap_lo]`. For a gap
// `[v_{t-1}, v_t]` of length `dv = v_t - v_{t-1}` between consecutive sorted
// endpoints:
//   - `p` pancakes have `b_i <= v_{t-1}` (both choices are `<= v_{t-1}`),
//   - `q` pancakes straddle (`a_i <= v_{t-1} < b_i`: `a_i` lands `<=`, `b_i` `>`),
//   - `n - p - q` have `a_i > v_{t-1}` (both choices are `> v_{t-1}`).
//
// Across all `2^n` assignments, `2^(n - q) * C(q, j)` assignments have exactly
// `p + j` values `<= v_{t-1}`; those are the assignments where the `k`-th smallest
// value exceeds `v_{t-1}` for every `k >= p + j`. Equivalently, the difference
// sequence `d[k] = ans[k] - ans[k - 1]` has generating function
//   `G(x) = sum_t dv_t * 2^(n - q_t) * x^(p_t) * (1 + x)^(q_t)`.
//
// Since `p_t` is non-decreasing along the sweep, divide-and-conquer merges a
// segment into `Segment(poly, p_min, q_min)` representing
// `x^(p_min) * (1 + x)^(q_min) * poly(x)`: lifting whichever half has larger
// `q_min` by `(1 + x)^(q_half - q_min)` via NTT convolution and shifting the
// right half by `rp_min - lp_min` computes `G(x)` in `O(n log^2 n)`. Prefix
// sums of `G` starting from `2^n * events[0].val` yield `ans[0..n]`.

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
