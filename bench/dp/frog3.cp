use std.dp.{LiChao, Line}
let (n, c): (2..=2e5, 1..=1e12) = read()
let h: [0..n]1..=1e6 = read()
var hull = LiChao<Line, Int>(min, 1..=10 ** 6)
var cost = 0
for x in h:
    if let best = hull.at(x): cost = best + x * x + c
    hull.add(Line(-2 * x, cost + x * x))
print(cost)
