// Codeforces 2138 F: Ode to the Lily.
// Triangle ladder from (0,0), (1,0) to t=(p,q) in m steps, edges in [0.5, 1].
// Even m=2k: k rungs d=t/k, side rail shifted by (0.5, d.y-0.5).
// Odd m=2k+1: unit rungs d=t.unit to t; if t0 fails p2, fallback is sqrt(3/4) from p2
// and valid to t1; the five annuli hold a disk of radius > 1/64, so a 1/64 grid finds s.

use std.geometry.{Vec}

fn fits(a: Vec, b: Vec, eps: F64 = 1e-9) -> Bool = 0.5 + eps <= (a - b).len <= 1.0 - eps

@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = Vec(F64(p), F64(q))
let k = m / 2
print(m)
var drawn = 2
fn draw(u: Int, v: Int, c: Vec) -> Int:
    print("{u} {v} {c.x:.20f} {c.y:.20f}")
    drawn += 1
    drawn

fn ladder(off: Int, on: Int, from: Int, off_at: Int -> Vec, on_at: Int -> Vec):
    var (a, b) = (off, on)
    for i in from..=k:
        a = draw(a, b, off_at(i))
        b = draw(a, b, on_at(i))

let (p1, p2) = (Vec(0.0, 0.0), Vec(1.0, 0.0))
if m % 2 == 0:
    let d = t / F64(k)
    let side = Vec(0.5, d.y - 0.5)
    ladder(2, 1, 1, i => side + d * F64(i - 1), i => d * F64(i))
else:
    let d = t.unit
    let (t1, t0) = (t - d * F64(k - 1), t - d * F64(k))
    let start = if fits(t0, p2): t0 else: Vec(
        0.5 + 0.75 * sqrt(3.0 / 13.0),
        0.75 + sqrt(3.0 / 13.0) * 0.5,
    )
    fn side() -> Vec:
        for x in -64..=64:
            for y in -64..=64:
                let c = Vec(F64(x) / 64.0, F64(y) / 64.0)
                if [p1, p2, t0, t1, start].all(e => fits(c, e)): return c
        p1
    let s = side()
    let a = draw(1, 2, s)
    let b = draw(2, a, start)
    let c = draw(a, b, t1)
    ladder(a, c, 2, i => s + d * F64(i - 1), i => t0 + d * F64(i))
