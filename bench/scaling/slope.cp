// n slopes; an update that adds slopes is paired so the function keeps its size
use std.slope.{Concave, Convex}
let n: Int = read()
let reps = 1 << 17
var s = 0

for i in 0..reps:
    s += Concave.point(i).lo // timed: slope.Concave.point
    s += Convex.point(i).lo // timed: slope.Convex.point
for _ in 0..8:
    s += Concave.flat(0..n).hi // timed: slope.Concave.flat
    s += Convex.flat(0..n).hi // timed: slope.Convex.flat
// Storage grows by moving the slopes already there.
for _ in 0..90:
    var f = Concave.flat(0..n)
    f.reserve(2 * n) // timed: slope.Concave.reserve
    s += f.hi

var f = Concave.flat(0..n)
var g = Convex.flat(0..n)
for i in 0..reps:
    s += f.lo // timed: slope.Concave.lo
    s += f.hi // timed: slope.Concave.hi
    s += g.lo // timed: slope.Convex.lo
    s += g.hi // timed: slope.Convex.hi
    f.shift(i % 3 - 1) // timed: slope.Concave.shift
    g.shift(i % 3 - 1) // timed: slope.Convex.shift
    f.add_const(i % 5) // timed: slope.Concave.add_const
    g.add_const(i % 5) // timed: slope.Convex.add_const
    f.add_linear(i % 3 - 1) // timed: slope.Concave.add_linear
    g.add_linear(i % 3 - 1) // timed: slope.Convex.add_linear
    f.add_quadratic(x => 0 * x * x + x) // timed: slope.Concave.add_quadratic
    g.add_quadratic(x => 0 * x * x - x) // timed: slope.Convex.add_quadratic
    let p = f.lo + i * 7919 % n
    let q = g.lo + i * 7919 % n
    f.add_left(p) // timed: slope.Concave.add_left
    g.add_left(q) // timed: slope.Convex.add_left
    f.add_right(p) // timed: slope.Concave.add_right
    g.add_right(q) // timed: slope.Convex.add_right
    f.add_abs(p) // timed: slope.Concave.add_abs
    g.add_abs(q) // timed: slope.Convex.add_abs
    s += f.argmax().start // timed: slope.Concave.argmax
    s += g.argmin().start // timed: slope.Convex.argmin
for _ in 0..200:
    s += f.at(f.hi) // timed: slope.Concave.at [let: c = n; lo = 0]
    s += g.at(g.hi) // timed: slope.Convex.at [let: c = n; lo = 0]
    s += f.max() // timed: slope.Concave.max
    s += g.min() // timed: slope.Convex.min

// m (or k) slopes made to fall (or rise) before each call
var hf = Concave.flat(0..n)
var hg = Convex.flat(0..n)
for _ in 0..reps:
    hf.add_right(hf.hi - 4)
    hf.prefix_max() // timed: slope.Concave.prefix_max [fixed: m]
    hg.add_right(hg.hi - 4)
    hg.prefix_min() // timed: slope.Convex.prefix_min [fixed: m]
for _ in 0..100:
    f.add_left(f.lo + 4)
    f.suffix_max() // timed: slope.Concave.suffix_max [fixed: k]
    g.add_left(g.lo + 4)
    g.suffix_min() // timed: slope.Convex.suffix_min [fixed: k]

// each call adds |g| (or b - a) slopes; n / 4 calls take n slopes to 3n at every size
var fc = Concave.flat(0..n)
var gc = Convex.flat(0..n)
for _ in 0..n / 4:
    fc.max_convolve(Concave.flat(0..=4)) // timed: slope.Concave.max_convolve [fixed: g]
    gc.min_convolve(Convex.flat(0..=4)) // timed: slope.Convex.min_convolve [fixed: g]
for _ in 0..n / 4:
    fc.slide_max(-2, 2) // timed: slope.Concave.slide_max [fixed: a, b]
    gc.slide_min(-2, 2) // timed: slope.Convex.slide_min [fixed: a, b]
for _ in 0..n / 4:
    fc.slide_max(-2, 2) // timed: slope.Concave.slide_max [fixed: a, b]
    gc.slide_min(-2, 2) // timed: slope.Convex.slide_min [fixed: a, b]
print(s)
