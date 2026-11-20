// NOI '08 P5 (Clouds): dynamic bouncing cloud segments on [0, m] queried for overlap
// with [l, r] at time t. Unfolds periodic bounces to period 2m in two rotated coordinate
// frames tracked by Fenwick2D, relying on v2.2 Euclidean `%` (non-negative for negative
// dividends) and Range.clamp to eliminate manual mod() and 4-way bounds clamping.
// original: corpus/submissions/6067573.cpp


let (q, m): (Int, Int) = read()
let p = 2 * m
domain Grid = 0..(2 * p)

struct BouncingBIT:
    bit: Fenwick2D<Grid, Grid, Int>

    fn new() -> BouncingBIT = BouncingBIT(Fenwick2D<Grid, Grid, Int>())

    fn add(mut self, x: Int, y: Int, d: Int):
        self.bit.add(Grid(x), Grid(y), d)

    fn rect(self, xl: Int, xr: Int, yl: Int, yr: Int) -> Int:
        fn clamped(x1: Int, x2: Int, y1: Int, y2: Int) -> Int:
            let rx = (x1..=x2).clamp(Grid)
            let ry = (y1..=y2).clamp(Grid)
            self.bit.sum(rx, ry)
        clamped(xl, xr, yl, yr) + clamped(xl - p, xr - p, yl - p, yr - p)

var ds1 = BouncingBIT.new()
var ds2 = BouncingBIT.new()
var clouds = Map<Int, (Int, Int)>()

for _ in 0..q:
    let ty: Int = read()
    if ty == 1:
        let (t, c, l, r, d): (Int, Int, Int, Int, Int) = read()
        let len = r - l
        let raw = if d == -1: p - l else: l
        let l_pos = (raw - t) % p
        ds1.add(l_pos, l_pos + len, 1)
        ds2.add(l_pos + p, l_pos - len + p, 1)
        clouds[c] = (l_pos, len)
    elif ty == 2:
        let (t, l, r): (Int, Int, Int) = read()
        let base = (-t) % p
        let a1 = ds1.rect(base, base + r, base + l, 3 * p - 1)
        let b2 = base + p
        let a2 = ds2.rect(b2 + max(r + 1, p - r), b2 + p - 1, 0, b2 + p - l)
        print(a1 + a2)
    else:
        let (_t, c): (Int, Int) = read()
        let (l_pos, len) = clouds[c]
        ds1.add(l_pos, l_pos + len, -1)
        ds2.add(l_pos + p, l_pos - len + p, -1)
