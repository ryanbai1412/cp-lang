// A ladder of triangles between two rails: each new point joins the two rail tips.
// Even m: m/2 rungs of t/(m/2). Odd m: three opening points reach the line through t,
// then unit rungs along it.
struct V(x: F64, y: F64):
    @get fn len(self) -> F64 = sqrt(x * x + y * y)

impl Add for V:
    fn add(a, b) = V(a.x + b.x, a.y + b.y)

impl Sub for V:
    fn sub(a, b) = V(a.x - b.x, a.y - b.y)

fn times(a: V, k: F64) -> V = V(a.x * k, a.y * k)

fn over(a: V, k: F64) -> V = V(a.x / k, a.y / k)

fn fits(a: V, b: V) -> Bool = 0.5 <= (a - b).len <= 1.0

@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = V(F64(p), F64(q))
let k = m / 2
print(m)
var drawn = 2
fn draw(u: Int, v: Int, c: V) -> Int:
    print("{u} {v} {c.x:.20f} {c.y:.20f}")
    drawn += 1
    drawn

fn ladder(off: Int, on: Int, from: Int, off_at: Int -> V, on_at: Int -> V):
    var (a, b) = (off, on)
    for i in from..=k:
        a = draw(a, b, off_at(i))
        b = draw(a, b, on_at(i))

let (p1, p2) = (V(0.0, 0.0), V(1.0, 0.0))
if m % 2 == 0:
    let d = over(t, F64(k))
    let side = V(0.5, d.y - 0.5)
    ladder(2, 1, 1, i => side + times(d, F64(i - 1)), i => times(d, F64(i)))
else:
    let d = over(t, t.len)
    let (t1, t0) = (t - times(d, F64(k - 1)), t - times(d, F64(k)))
    let start = if fits(t0, p2): t0 else: V(
        0.5 + 0.75 * sqrt(3.0 / 13.0),
        0.75 + sqrt(3.0 / 13.0) * 0.5,
    )
    fn side() -> V:
        for x in -64..=64:
            for y in -64..=64:
                let c = V(F64(x) / 64.0, F64(y) / 64.0)
                let near = fits(c, p1) and fits(c, p2) and fits(c, start)
                if near and fits(c, t0) and fits(c, t1): return c
        p1
    let s = side()
    let a = draw(1, 2, s)
    let b = draw(2, a, start)
    let c = draw(a, b, t1)
    ladder(a, c, 2, i => s + times(d, F64(i - 1)), i => t0 + times(d, F64(i)))
