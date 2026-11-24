// Source: AtCoder ABC481 G (Half-Baked), https://atcoder.jp/contests/abc481/tasks/abc481_g
// Problem: n <= 120000 pancakes, pancake i has size a_i or b_i (a_i < b_i < 998244353) with
//   probability 1/2 each. For each k print 2^n E[k-th smallest size] mod 998244353.
// Solution: write the k-th smallest as v_min plus each gap dv between consecutive sorted
//   endpoints for which it exceeds the gap's low end. If p pancakes lie wholly below a gap
//   and q straddle it, 2^(n-q) C(q, j) choices put exactly p + j values below, so
//   ans[k] - ans[k-1] = [x^k] sum_t dv_t 2^(n-q_t) x^(p_t) (1 + x)^(q_t). Divide and conquer
//   over the gaps, factoring out x^p_min (1 + x)^q_min, with NTT products (Poly *): O(n log^2 n).

use std.poly.{Poly}

type Z = Mod<998_244_353>

enum End: Low | High
struct Event(val: Int, end: End)
struct Gap(p: Int, q: Int, w: Z)
// poly * x^p_min * (1 + x)^q_min
struct Segment(poly: Poly<Z>, p_min: Int, q_min: Int)

let n: 1..=120_000 = read()
domain I = 0..n
let cakes: [I](a: 1..=998_244_352, b: 1..=998_244_352) = read()

let comb = Comb<Z>(n)

// (1 + x)^d
fn binom(d: Int) -> Poly<Z> = Poly([comb.choose(d, i) for i in 0..=d])
// f * x^k
fn times_x(f: Poly<Z>, k: Int) -> Poly<Z> = Poly(List.filled(k, Z(0)) + f.c)

var events: List<Event> = []
for i in I:
    events.push(Event(cakes[i].a, End.Low))
    events.push(Event(cakes[i].b, End.High))
events.sort(by: e => (e.val, e.end))

var gaps: List<Gap> = []
var cur_p = 0
var cur_q = 0
for (prev, cur) in events.adjacent:
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
        return Segment(Poly([gaps[l].w]), gaps[l].p, gaps[l].q)
    let mid = (l + r) / 2
    let (left, right) = (merge_gaps(l, mid), merge_gaps(mid, r))
    let q_min = min(left.q_min, right.q_min)
    let lp = left.poly * binom(left.q_min - q_min)
    let rp = right.poly * binom(right.q_min - q_min)
    Segment(lp + times_x(rp, right.p_min - left.p_min), left.p_min, q_min)

let top = merge_gaps(0, gaps.len)
let diff = times_x(top.poly * binom(top.q_min), top.p_min)

let base = (Z(2) ** n) * Z(events[0].val)
let delta: [I]Z = [I] i => (if i == I.first: base else: Z(0)) + diff.at(i.ord)
let running = delta.scan(+)
print([running.prefix(i.r) for i in I], sep: "\n")

@alt(program)
fn slow():
    var tot: [I]Z = Z(0)
    for mask in 0..(1 << n):
        let vals = [if not mask.bit(i.ord): cakes[i].a else: cakes[i].b for i in I].sorted()
        for i in I:
            tot[i] += Z(vals[i.ord])
    print(tot, sep: "\n")
