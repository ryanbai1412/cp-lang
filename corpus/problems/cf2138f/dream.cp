// Codeforces 2138 F: Triangle Strip.
// Grow a strip of triangles from the base segment `p1 = (0, 0)`, `p2 = (1, 0)`
// to `t = (p, q)`: each new point joins two earlier points with both edge
// lengths in `[1/2, 1]`.
//   - Even `m = 2k`: a ladder of `k` rungs stepping by `d = t / k`, with the
//     off-rail shifted to `(0.5, d.y - 0.5)`.
//   - Odd `m = 2k + 1`: three opening steps bring the strip onto the ray to `t`
//     at `t1 = t - (k - 1) * t.unit`, followed by `k - 1` unit rungs.
//     `start` is `t0 = t - k * t.unit` when `fits(t0, p2)`, else the upper
//     intersection of the unit circles centered at `p2 = (1, 0)` and `(0, 1.5)`
//     (midpoint `(0.5, 0.75)` plus perpendicular `(0.75, 0.5) * sqrt(3 / 13)`).
//     Because the 5 annuli `0.5 <= |c - e| <= 1` around `[p1, p2, t0, t1, start]`
//     always contain a disc of radius > 1/32 inside `[-1, 1]^2`, the 1/64 grid
//     is guaranteed to contain a valid bridge point `s`.

use std.geometry.Vec

@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = Vec(p, q)
let k = m / 2
let (p1, p2) = (Vec(0, 0), Vec(1, 0))
fn fits(a: Vec, b: Vec) -> Bool = 0.5 <= (a - b).len <= 1.0

struct Step(u: Vec, v: Vec, at: Vec)

fn ladder(off0: Vec, on0: Vec, rungs: List<(off: Vec, on: Vec)>) -> List<Step>:
    var (off_tip, on_tip) = (off0, on0)
    var out: List<Step> = []
    for r in rungs:
        out.push(Step(off_tip, on_tip, r.off))
        off_tip = r.off
        out.push(Step(off_tip, on_tip, r.on))
        on_tip = r.on
    out

let steps = if m % 2 == 0:
    let d = t / k
    let side = Vec(0.5, d.y - 0.5)
    ladder(p2, p1, [(side + d * (i - 1), d * i) for i in 1..=k])
else:
    let d = t.unit
    let (t1, t0) = (t - d * (k - 1), t - d * k)
    let fallback_start = Vec(0.5, 0.75) + Vec(0.75, 0.5) * sqrt(3.0 / 13.0)
    let start = if fits(t0, p2): t0 else: fallback_start
    let s = [Vec(x, y) / 64 for x in -64..=64 for y in -64..=64]
        .first(c => [p1, p2, t0, t1, start].all(e => fits(c, e)))!
    let opening = [Step(p1, p2, s), Step(p2, s, start), Step(s, start, t1)]
    opening + ladder(s, t1, [(s + d * (i - 1), t0 + d * i) for i in 2..=k])

var id: Map<Vec, Int> = [ p1: 1, p2: 2 ]
print(m)
for st in steps:
    print("{id[st.u]!} {id[st.v]!} {st.at.x:.20f} {st.at.y:.20f}")
    id[st.at] = id.len + 1
