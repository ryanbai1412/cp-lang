// expect: ok, output "-1\n1 1 -4\nYES"
use std.dp.{LiChao, Line}
use std.rand.{Rng}
var t = LiChao<Line, Int>(min, over: 0..=10)
print(t.at(5) ?? -1)
t.add(Line(1, 0))
t.add(Line(-1, 6))
print(t.at(1) ?? -1, t.at(5) ?? -1, t.at(10) ?? -1)
var rng = Rng(2)
var u = LiChao<Line, Int>(max, over: -50..50)
var ls: List<(Int, Int)> = []
var ok = true
for _ in 0..100:
    let (k, m) = (rng.int(-9, 9), rng.int(-99, 99))
    u.add(Line(k, m))
    ls.push((k, m))
    let x = rng.int(-50, 49)
    var best = -1_000_000
    for (a, b) in ls: best = max(best, a * x + b)
    if u.at(x) != Some(best): ok = false
print(ok)
