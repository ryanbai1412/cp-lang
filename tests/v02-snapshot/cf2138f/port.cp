// Source: Codeforces 2138 F (Ode to the Bridge Builder),
//   https://codeforces.com/contest/2138/problem/F
// Problem: start from points (0, 0), (1, 0) joined by a segment; an operation picks a
//   segment AB and adds a point C with triangle ABC's sides all in [1/2, 1]. Reach integer
//   (p, q) (1 <= p, q <= 1e4) within 1e-4 using at most m = ceil(2 |(p, q)|) operations; print
//   them. Multitest, sum of m <= 1e5.
// Solution: build a ladder of triangles toward t = (p, q), two operations per rung. Even
//   m = 2k: k rungs stepping by t / k (length <= 1), the second rail shifted to
//   (0.5, d.y - 0.5). Odd m = 2k + 1: three opening operations reach t - (k - 1) t.unit on
//   the ray, then k - 1 unit rungs; the bridge point is taken from the 1/64 grid in [-1, 1]^2,
//   which always holds a point within [1/2, 1] of the five anchors. O(m) per test.

use std.geometry.Vec

@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = Vec(F64(p), F64(q))
let k = m / 2
let (p1, p2) = (Vec(0.0, 0.0), Vec(1.0, 0.0))
fn fits(a: Vec, b: Vec, eps: F64 = 1e-9) -> Bool = 0.5 + eps <= (a - b).len <= 1.0 - eps

struct Step(u: Int, v: Int, at: Vec)

fn ladder(off0: Int, on0: Int, next0: Int, rungs: List<(off: Vec, on: Vec)>) -> List<Step>:
    var (off_tip, on_tip, nxt) = (off0, on0, next0)
    var out: List<Step> = []
    for r in rungs:
        out.push(Step(off_tip, on_tip, r.off))
        off_tip = nxt
        nxt += 1
        out.push(Step(off_tip, on_tip, r.on))
        on_tip = nxt
        nxt += 1
    out

let steps = if m % 2 == 0:
    let d = t / F64(k)
    let side = Vec(0.5, d.y - 0.5)
    ladder(2, 1, 3, [(side + d * F64(i - 1), d * F64(i)) for i in 1..=k])
else:
    let d = t.unit
    let (t1, t0) = (t - d * F64(k - 1), t - d * F64(k))
    let fallback_start = Vec(0.5, 0.75) + Vec(0.75, 0.5) * sqrt(3.0 / 13.0)
    let start = if fits(t0, p2): t0 else: fallback_start
    let s = [Vec(F64(x) / 64.0, F64(y) / 64.0) for x in -64..=64 for y in -64..=64]
        .find(c => [p1, p2, t0, t1, start].all(e => fits(c, e)))!
    let opening = [Step(1, 2, s), Step(2, 3, start), Step(3, 4, t1)]
    opening + ladder(3, 5, 6, [(s + d * F64(i - 1), t0 + d * F64(i)) for i in 2..=k])

print(m)
for st in steps:
    print("{st.u} {st.v} {st.at.x:.20f} {st.at.y:.20f}")
