use std.dp.{Line, LineContainer}
let n: Int = read()
let reps = 1 << 17
var s = 0
let xs = [i * 7919 % n for i in 0..n]

var hull = LineContainer(max)
for i in 0..reps:
    hull.add(Line(xs[i % n] - n / 2, xs[i * 3 % n])) // timed: dp.LineContainer.add
    s += hull.at(i * 13 % n) ?? 0 // timed: dp.LineContainer.at
    s += (hull.line_at(i * 17 % n) ?? Line(0, 0)).m // timed: dp.LineContainer.line_at
    s += hull.len // timed: dp.LineContainer.len
    if hull.is_empty: s += 1 // timed: dp.LineContainer.is_empty
var full = LineContainer(max)
for i in 0..n: full.add(Line(i, -i * i))
for _ in 0..40:
    s += full.each().count() // timed: dp.LineContainer.each
print(s)
