// Source: CCC 2000 S5 (Sheep and Coyotes), https://dmoj.ca/problem/ccc00s5
// limits: 1000 ms, 256 MB
// Problem: up to 1000 sheep in a 1000 x 1000 field, coordinates with two decimals. A coyote
//   enters anywhere on the south side (y = 0, 0 <= x <= 1000) and eats the nearest sheep (any of
//   tied ones). Print "The sheep at (x, y) might be eaten." for each sheep that can be eaten.
// Solution: p beats q at entry x iff 2x(q.x - p.x) <= |q|^2 - |p|^2, one half-line per rival, so
//   p's entry points are an interval; a rival with the same x and a lower y wins everywhere.
//   Coordinates are scaled to integers and interval ends kept as exact `Ratio`s. O(n^2).
// original: corpus/submissions/4584542.cpp
// Bounds: n: 1..=1000
const SCALE = 1000  // the coordinates have two decimals; a third keeps rounding exact

let n: 1..=1000 = read()
let read_at: [n](x: F64, y: F64) = read()  // 0 <= x, y <= 1000, two decimals
let sheep = [(x: round(s.x * SCALE), y: round(s.y * SCALE)) for s in read_at]

for p in sheep:
    if sheep.any(q => q.x == p.x and q.y < p.y): continue  // a rival straight below wins everywhere
    fn cut(q: (x: Int, y: Int)) -> Ratio = Ratio(q.x * q.x + q.y * q.y - p.x * p.x - p.y * p.y, 2 * (q.x - p.x))
    let lo = ([Ratio(0, 1)] + [cut(q) for q in sheep if q.x < p.x]).max()!
    let hi = ([Ratio(1000 * SCALE, 1)] + [cut(q) for q in sheep if q.x > p.x]).min()!
    if lo <= hi: print("The sheep at ({fixed(p.x, SCALE, 2)}, {fixed(p.y, SCALE, 2)}) might be eaten.")
