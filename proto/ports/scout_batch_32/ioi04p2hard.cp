// IOI '04 Hermes, large limits (DMOJ: ioi04p2hard)
// Ideal v2 port. original: corpus/submissions/4182973.cpp
//
// A messenger starts at (0, 0) and walks along grid lines. He must see n
// points in the given order; a point is seen from anywhere on its row or
// column. Print the shortest total walk.
//
// After seeing point i he stands on its column or on its row, and the free
// coordinate is one of the earlier points' coordinates. The DP therefore asks
// "cheapest offer (q, c) measured as c + |p - q|", which splits into two
// prefix minima and needs an ordered map with `last_le`.
let n: 1..=2e5 = read()
let pts: [1..=n](x: 0..=1e9, y: 0..=1e9) = read()
let a: [0..=n]Int = [0..=n] i => if i == 0: 0 else: pts[i].x
let b: [0..=n]Int = [0..=n] i => if i == 0: 0 else: pts[i].y
let walk_a = prefix(abs(a[i] - a[i + 1]) for i in 0..n)
let walk_b = prefix(abs(b[i] - b[i + 1]) for i in 0..n)

// Minimum value over all keys <= x. Only entries that improve on every
// smaller key are kept, so the answer is the entry found by `last_le`.
struct MinUpTo:
    m: Map<Int, Int> = Map()
    fn offer(mut self, k: Int, v: Int):
        if let (_, best) = m.last_le(k) and best <= v: return
        // WISH X2: `Set` has `first_gt`, the ordered `Map` only `first_ge`.
        while let (k2, v2) = m.first_ge(k + 1) and v2 > v: m.remove(k2)
        m[k] = v
    fn upto(self, x: Int) -> ?Int = if let (_, v) = m.last_le(x): v else: none

// Cheapest c + |p - q| over the offers (q, c) made so far.
struct Cheapest:
    below: MinUpTo = MinUpTo()
    above: MinUpTo = MinUpTo()
    fn offer(mut self, q: Int, c: Int):
        below.offer(q, c - q)
        above.offer(-q, c + q)
    fn at(self, p: Int) -> ?Int:
        // WISH X1: `min(below.upto(p) + p, above.upto(-p) - p)` is the formula.
        var best: ?Int = none
        if let v = below.upto(p): best min= v + p
        if let v = above.upto(-p): best min= v - p
        best

var (by_y, by_x) = (Cheapest(), Cheapest())
fn offer(i: Int, da: Int, db: Int):
    if i == 0: return
    by_y.offer(b[i], walk_a.sum(..i - 1) + da)
    by_x.offer(a[i], walk_b.sum(..i - 1) + db)

var (da, db) = (0, 0)
offer(n, 0, 0)
for i in (0..n).rev:
    let (pa, pb) = if i > 0: (a[i - 1], b[i - 1]) else: (0, 0)
    (da, db) = (walk_b.sum(i..n), walk_a.sum(i..n))
    if let v = by_x.at(pa): da min= v - walk_b.sum(..i)
    if let v = by_y.at(pb): db min= v - walk_a.sum(..i)
    offer(i, da, db)
print(min(da, db))
