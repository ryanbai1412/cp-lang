use std.dp.{Line, LiChao}
let n: Int = read()
let reps = 1 << 17
var s = 0
let xs = [i * 7919 % n for i in 0..n]

for i in 0..reps:
    s += Line(i % 7, i % 5).at(i % n) // timed: dp.Line.at
for _ in 0..reps:
    var e = LiChao(min, over: 0..=n) // timed: dp.LiChao.new
    s += 1
var lc = LiChao(min, over: 0..=n)
for i in 0..reps:
    lc.add(Line(i % 101 - 50, xs[i % n])) // timed: dp.LiChao.add
    s += lc.at(i * 13 % n) ?? 0 // timed: dp.LiChao.at
    s += (lc.line_at(i * 17 % n) ?? Line(0, 0)).k // timed: dp.LiChao.line_at
for i in 0..reps:
    var c = LiChao(min, over: 0..=n)
    c.add(Line(1, i))
    c.clear() // timed: dp.LiChao.clear
print(s)
