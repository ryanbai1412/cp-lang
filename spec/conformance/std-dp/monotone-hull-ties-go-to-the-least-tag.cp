// expect: ok, output "1 1\n0 2"
// Three lines meeting at x = 1: the middle one touches the envelope only there, and its tag is the
// least, so it stays and wins the tie. An equal line keeps the smaller tag, as `aliens` needs (ioi16p6).
var h = MonotoneHull<Int>(min)
h.add(Line(1, 0), tag: 5)
h.add(Line(0, 1), tag: 1)
h.add(Line(-1, 2), tag: 9)
let b = h.best_next(1)!
print(b.val, b.tag)
var g = MonotoneHull<Int>(min)
g.add(Line(0, 0), tag: 4)
g.add(Line(0, 0), tag: 2)
let c = g.best_next(0)!
print(c.val, c.tag)
