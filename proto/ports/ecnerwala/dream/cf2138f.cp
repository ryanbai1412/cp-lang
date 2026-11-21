use std.geometry.{Vec}

// Codeforces 2138 F (dream). Grow a strip of triangles from the unit segment
// (0,0)-(1,0) to t = (p, q): every new point joins two earlier points, and every
// edge has length in [1/2, 1].
// Even m: a ladder of m/2 rungs, each step d = t/(m/2); the off-rail points sit
// at side + d·(i − 1). Odd m: three opening points bring the strip to the line
// through t, then unit rungs along that line. The opening point `start` is t0
// when it is within reach of (1,0), else a fixed point at distance 1 from (1,0);
// the bridge point s is any point of the 1/64 grid within reach of all five
// fixed points, which always exists.
@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = Vec(p, q)
let k = m / 2
fn fits(a: Vec, b: Vec) -> Bool = 0.5 <= (a - b).len <= 1.0

struct Step(u: Int, v: Int, at: Vec)             // ids are 1-based; 1 and 2 are given

// Rungs between an off rail (tip a) and an on rail (tip b); the steps get ids from `next`.
fn ladder(a: Int, b: Int, rungs: List<(off: Vec, on: Vec)>, next: Int) -> List<Step>:
    var (off_tip, on_tip) = (a, b)
    var steps: List<Step> = []
    for (j, r) in rungs.items:
        steps.push(Step(off_tip, on_tip, r.off))
        off_tip = next + 2 * j
        steps.push(Step(off_tip, on_tip, r.on))
        on_tip = off_tip + 1
    steps

let (p1, p2) = (Vec(0, 0), Vec(1, 0))
let steps =
    if m % 2 == 0:
        let d = t / k
        let side = Vec(0.5, d.y - 0.5)
        ladder(2, 1, [(side + d * (i - 1), d * i) for i in 1..=k], next: 3)
    else:
        let d = t.unit
        let (t1, t0) = (t - d * (k - 1), t - d * k)
        let start = if fits(t0, p2): t0 else: Vec(0.5 + 0.75 * sqrt(3 / 13), 0.75 + sqrt(3 / 13) / 2)
        let s = [Vec(x, y) / 64 for x in -64..=64 for y in -64..=64]
            .first(c => [p1, p2, t0, t1, start].all(e => fits(c, e)))!
        let opening = [Step(1, 2, s), Step(2, 3, start), Step(3, 4, t1)]
        opening + ladder(3, 5, [(s + d * (i - 1), t0 + d * i) for i in 2..=k], next: 6)

print(m)
for st in steps: print("{st.u} {st.v} {st.at.x:.20f} {st.at.y:.20f}")
