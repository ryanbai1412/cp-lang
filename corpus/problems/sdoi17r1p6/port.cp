// Source: SDOI 2017 Round 1 P6 (Correlation Analysis, Luogu P3707), https://dmoj.ca/problem/sdoi17r1p6
// limits: 1000 ms, 125 MB
// Problem: arrays x, y of length n and m ops on l..=r: 1 l r prints the least-squares slope
//   sum (x - mx)(y - my) / sum (x - mx)^2 (mx, my the means; error <= 1e-5); 2 l r S T adds S to
//   every x and T to every y; 3 l r S T sets x[i] = S + i, y[i] = T + i. n, m <= 1e5;
//   |x|, |y|, |S|, |T| <= 1e5.
// Solution: times the length k, the slope is (k sum xy - sum x sum y) / (k sum x^2 - (sum x)^2),
//   so a lazy segment tree keeps the four sums per node, plus where it starts, because "set"
//   writes a different value at each i. Set-then-add tags compose, and each updates the sums
//   in closed form: O(log n) per op. Adds pile up to 1e10 per value, so the sums are I128.
// original: corpus/submissions/5461671.cpp
// Bounds: n, m: 1..=1e5; x, y, s, t: -1e5..=1e5; kind: 1..=3; l, r: 1..=n
struct Sums(xy: I128, x: I128, y: I128, xx: I128, first: Int)
struct Edit(set: ?(s: Int, t: Int), dx: Int, dy: Int)  // set first, then add

/// 1 + ... + k and 1^2 + ... + k^2.
fn tri(k: Int) -> I128 = I128(k) * (k + 1) / 2
fn pyr(k: Int) -> I128 = I128(k) * (k + 1) * (2 * k + 1) / 6

fn join(a: Sums, b: Sums) -> Sums = Sums(a.xy + b.xy, a.x + b.x, a.y + b.y, a.xx + b.xx, min(a.first, b.first))

fn edit(f: Edit, v: Sums, len: Int) -> Sums:
    let k = I128(len)
    var s = v
    if let (sx, ty) = f.set:
        // x[i] = sx + i and y[i] = ty + i over i in first..first + len
        let (a, b) = (I128(sx), I128(ty))
        let i1 = tri(v.first + len - 1) - tri(v.first - 1)
        let i2 = pyr(v.first + len - 1) - pyr(v.first - 1)
        s = Sums(k * a * b + (a + b) * i1 + i2, k * a + i1, k * b + i1, k * a * a + 2 * a * i1 + i2, v.first)
    let (dx, dy) = (I128(f.dx), I128(f.dy))
    Sums(s.xy + dx * s.y + dy * s.x + k * dx * dy, s.x + k * dx, s.y + k * dy, s.xx + 2 * dx * s.x + k * dx * dx, s.first)

/// `f` after `g`.
fn then(f: Edit, g: Edit) -> Edit = if f.set != None: f else: Edit(g.set, g.dx + f.dx, g.dy + f.dy)

let (n, m): (1..=1e5, 1..=1e5) = read()
domain Pos = 1..=n
let x: [Pos]-1e5..=1e5 = read()
let y: [Pos]-1e5..=1e5 = read()
let edits = Action<Sums, Edit>(
    Edit(None, 0, 0),
    then,
    edit,
    over: Monoid<Sums>(Sums(0, 0, 0, 0, n + 1), join),
)
var tree = SegTree(
    edits.over,
    [Pos] i => Sums(I128(x[i]) * y[i], I128(x[i]), I128(y[i]), I128(x[i]) * x[i], i.label),
    act: edits,
)

for _ in 0..m:
    let (kind, l, r): (1..=3, Pos, Pos) = read()
    if kind == 1:
        let v = tree.fold(l..=r)
        let k = I128(r.label - l.label + 1)
        print(F64(k * v.xy - v.x * v.y) / F64(k * v.xx - v.x * v.x), digits: 5)
    else:
        let (s, t): (-1e5..=1e5, -1e5..=1e5) = read()
        tree.apply(l..=r, if kind == 2: Edit(None, s, t) else: Edit((s, t), 0, 0))
