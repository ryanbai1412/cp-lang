// n slopes; an update that adds slopes is paired so the function keeps its size
// n: 2048
use std.slope.{Concave, Convex}
let n: Int = read()
let reps = 1 << 17
var s = 0

for i in 0..reps:
    s += Concave.point(i).lo  // timed: slope.Concave.point
    s += Convex.point(i).lo   // timed: slope.Convex.point
for _ in 0..64:
    s += Concave.flat(0..n).hi  // timed: slope.Concave.flat
    s += Convex.flat(0..n).hi   // timed: slope.Convex.flat
// Storage grows by moving the slopes already there. At n / 128 slopes the doubled block stays
// under 128 KB at 4n too, so malloc serves both sizes from the heap and neither pays fresh-page
// faults that the other doesn't.
for _ in 0..12000:
    var f = Concave.flat(0..n / 128)
    f.reserve(n / 64)  // timed: slope.Concave.reserve
    s += f.hi

var f = Concave.flat(0..n)
var g = Convex.flat(0..n)
for i in 0..reps:
    s += f.lo                            // timed: slope.Concave.lo
    s += f.hi                            // timed: slope.Concave.hi
    s += g.lo                            // timed: slope.Convex.lo
    s += g.hi                            // timed: slope.Convex.hi
    f.shift(i % 3 - 1)                   // timed: slope.Concave.shift
    g.shift(i % 3 - 1)                   // timed: slope.Convex.shift
    f.add_const(i % 5)                   // timed: slope.Concave.add_const
    g.add_const(i % 5)                   // timed: slope.Convex.add_const
    f.add_linear(i % 3 - 1)              // timed: slope.Concave.add_linear
    g.add_linear(i % 3 - 1)              // timed: slope.Convex.add_linear
    f.add_quadratic(x => 0 * x * x + x)  // timed: slope.Concave.add_quadratic
    g.add_quadratic(x => 0 * x * x - x)  // timed: slope.Convex.add_quadratic
    let p = f.lo + i * 7919 % n
    let q = g.lo + i * 7919 % n
    f.add_left(p)            // timed: slope.Concave.add_left
    g.add_left(q)            // timed: slope.Convex.add_left
    f.add_right(p)           // timed: slope.Concave.add_right
    g.add_right(q)           // timed: slope.Convex.add_right
    f.add_abs(p)             // timed: slope.Concave.add_abs
    g.add_abs(q)             // timed: slope.Convex.add_abs
    s += f.argmax().start    // timed: slope.Concave.argmax
    s += g.argmin().start    // timed: slope.Convex.argmin
    s += f.at(f.hi - i % 7)  // timed: slope.Concave.at
    s += g.at(g.hi - i % 7)  // timed: slope.Convex.at
    s += f.max()             // timed: slope.Concave.max
    s += g.min()             // timed: slope.Convex.min

// a few slopes made to fall (or rise) before each call
var hf = Concave.flat(0..n)
var hg = Convex.flat(0..n)
for _ in 0..reps:
    hf.add_right(hf.hi - 4)
    hf.prefix_max()  // timed: slope.Concave.prefix_max
    hg.add_right(hg.hi - 4)
    hg.prefix_min()  // timed: slope.Convex.prefix_min
// Every call walks the front of one tree, whose depth is one draw of a random variable: trees of
// sizes n + j * n / 16 have fronts drawn from different priorities, so cycling over them times the
// expected depth.
var sf = (0..8).map(j => Concave.flat(0..n + j * n / 16)).to_list()
var sg = (0..8).map(j => Convex.flat(0..n + j * n / 16)).to_list()
for i in 0..reps:
    let j = i % 8
    sf[j].add_left(sf[j].lo + 4)
    sf[j].suffix_max()  // timed: slope.Concave.suffix_max
    sg[j].add_left(sg[j].lo + 4)
    sg[j].suffix_min()  // timed: slope.Convex.suffix_min

// each call adds |g| (or b - a) slopes; n / 4 calls take n slopes to 3n at every size
var fc = Concave.flat(0..n)
var gc = Convex.flat(0..n)
for _ in 0..n / 4:
    fc.max_convolve(Concave.flat(0..=4))  // timed: slope.Concave.max_convolve [fixed: g]
    gc.min_convolve(Convex.flat(0..=4))   // timed: slope.Convex.min_convolve [fixed: g]
for _ in 0..n / 4:
    fc.slide_max(-2, 2)  // timed: slope.Concave.slide_max [fixed: a, b]
    gc.slide_min(-2, 2)  // timed: slope.Convex.slide_min [fixed: a, b]
for _ in 0..n / 4:
    fc.slide_max(-2, 2)  // timed: slope.Concave.slide_max [fixed: a, b]
    gc.slide_min(-2, 2)  // timed: slope.Convex.slide_min [fixed: a, b]
print(s)
