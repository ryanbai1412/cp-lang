use std.geometry.{Vec}

// A ladder of triangles between two rails: each new point joins the two rail tips.
// Even m: m/2 rungs of t/(m/2). Odd m: three opening points reach the line through t,
// then unit rungs along it.
@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = Vec(p, q)
let k = m / 2
fn fits(a: Vec, b: Vec) -> Bool = 0.5 <= (a - b).len <= 1.0

print(m)
var pts: List<Vec> = [Vec(0, 0), Vec(1, 0)]     // ids are the 1-based list positions
fn draw(u: Int, v: Int, c: Vec) -> Int:
    print("{u} {v} {c.x:.20f} {c.y:.20f}")
    pts.push(c)                                 // returns the new id

fn ladder(a: Int, b: Int, from: Int, off_at: Int -> Vec, on_at: Int -> Vec):
    for i in from..=k:
        a = draw(a, b, off_at(i))
        b = draw(a, b, on_at(i))

let (p1, p2) = (pts[1], pts[2])
if m % 2 == 0:
    let d = t / k
    let side = Vec(0.5, d.y - 0.5)
    ladder(2, 1, 1, i => side + d * (i - 1), i => d * i)
else:
    let d = t.unit
    let (t1, t0) = (t - d * (k - 1), t - d * k)
    let start = if fits(t0, p2): t0 else: Vec(0.5 + 0.75 * sqrt(3 / 13), 0.75 + sqrt(3 / 13) / 2)
    let s = (-64..=64).pairs(-64..=64).map((x, y) => Vec(x, y) / 64)
        .find(c => [p1, p2, t0, t1, start].all(e => fits(c, e)))
        ?? unreachable("some grid point is within reach of all five")
    let a = draw(1, 2, s)
    let b = draw(2, a, start)
    let c = draw(a, b, t1)
    ladder(a, c, 2, i => s + d * (i - 1), i => t0 + d * i)
