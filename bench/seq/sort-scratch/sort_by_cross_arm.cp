struct Dir(d: Pt)

impl Ord for Dir:
    fn lt(a, b) = cross(a.d, b.d) > 0

impl Eq for Dir:
    fn eq(a, b) = cross(a.d, b.d) == 0

let n = 2500
var s = 88_172_645
fn next() -> Int:
    s = (s * 1_103_515_245 + 12_345) % 2_147_483_648
    s % 1_000_000_000
let pts = [Pt(next(), next()) for _ in 0..n].sorted(by: p => (p.x, p.y))
var turns: List<(d: Pt, a: Int, b: Int)> = []
turns.reserve(n * (n - 1) / 2)
for i in 0..n:
    for j in i + 1..n: turns.push((pts[j] - pts[i], i, j))
turns.sort(by: t => Dir(t.d))
var acc = 0
for t in turns: acc = (acc * 31 + t.a * n + t.b) % 1_000_000_007
print(acc)
