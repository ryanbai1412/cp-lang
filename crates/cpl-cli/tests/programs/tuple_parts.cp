// Tuples a loop takes apart, in both engines: a yields fn's tuple of
// scalars and of ranges handed over part by part, a body that also takes
// the tuple whole, and a tuple pattern over a comprehension's splits.
fn pairs(n: Int) -> yields (Int, Int):
    for i in 0..n:
        yield (i, i * i - 3 * i)

fn halves(r: Range<Int>) -> yields (Range<Int>, Range<Int>):
    for (x, y) in r.splits:
        yield (y, x)

var t = 0
for (i, v) in pairs(6):
    t = t * 7 + i * v
print(t)
var best = (0, 0)
for p in pairs(6):
    if p.1 < best.1: best = p
print(best.0, best.1)
var s = 0
for (x, y) in halves(0..5):
    s = s * 3 + x.len * 10 + y.len
print(s)
print(min(x.len * 2 + y.len for (x, y) in (2..9).splits)!)
