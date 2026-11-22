// expect: ok, output "0 1 -2 2\n0 1\nYES\nYES\nYES"
use std.dp.{MonotoneHull, Line}
use std.rand.{Rng}
var h = MonotoneHull<Int>(min)
h.add(Line(2, 0), tag: 0)
h.add(Line(-1, 3), tag: 1)
print(h.at(0) ?? -1, h.at(2) ?? -1, h.at(5) ?? -1, h.len)
print(h.at_next(0) ?? -1, h.at_next(2) ?? -1)
var g = MonotoneHull<Int>(min)
g.add(Line(1, 0), tag: 7)
g.add(Line(-1, 2), tag: 3)
print(g.best_next(1) == Some((val: 1, tag: 3)))
h.clear()
print(h.at(0) == None)
var rng = Rng(9)
var ok = true
for _ in 0..200:
    let use_max = rng.int(0, 1) == 1
    var hh = if use_max: MonotoneHull<Int>(max) else: MonotoneHull<Int>(min)
    var ks = sorted([rng.int(-6, 6) for _ in 0..rng.int(1, 8)])
    if not use_max: ks = ks.reversed()
    var lines: List<Line> = []
    for k in ks:
        let l = Line(k, rng.int(-15, 15))
        lines.push(l)
        hh.add(l, tag: lines.len)
    var x = -10
    while x <= 10:
        var best = lines[0].k * x + lines[0].m
        for l in lines:
            let v = l.k * x + l.m
            if (use_max and v > best) or (not use_max and v < best): best = v
        if hh.at(x) != Some(best) or hh.at_next(x) != Some(best): ok = false
        x += rng.int(0, 3)
print(ok)
