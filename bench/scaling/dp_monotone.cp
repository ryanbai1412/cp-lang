use std.dp.{Line, MonotoneHull}
let n: Int = read()
let reps = 1 << 17
var s = 0

for _ in 0..40:
    var h = MonotoneHull<()>(min)
    for i in 0..n:
        h.add(Line(n - i, i * i % 1000)) // timed: dp.MonotoneHull.add
    for i in 0..n:
        s += h.at_next(i) ?? 0 // timed: dp.MonotoneHull.at_next
var mh = MonotoneHull<Int>(min)
// tangents of -x^2 at 0..n: every line is best somewhere, so all n stay
for i in 0..n: mh.add(Line(-2 * i, i * i), tag: i)
for i in 0..reps:
    s += mh.at(i * 7919 % n) ?? 0 // timed: dp.MonotoneHull.at
    s += mh.len // timed: dp.MonotoneHull.len
    if mh.is_empty: s += 1 // timed: dp.MonotoneHull.is_empty
for _ in 0..500:
    s += mh.items.count() // timed: dp.MonotoneHull.items
for i in 0..reps:
    var h = MonotoneHull<()>(min)
    h.add(Line(1, i))
    h.clear() // timed: dp.MonotoneHull.clear
for _ in 0..40:
    var h = MonotoneHull<Int>(min)
    for i in 0..n: h.add(Line(n - i, i), tag: i)
    for i in 0..n:
        s += (h.best_next(i) ?? (val: 0, tag: 0)).tag // timed: dp.MonotoneHull.best_next
print(s)
