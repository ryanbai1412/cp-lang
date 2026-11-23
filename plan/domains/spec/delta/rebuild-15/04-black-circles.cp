// CF 2002C Black Circles
// in: t; per test n, n lines x y, then xs ys xt yt. out: YES if the goal is reachable, else NO.
fn dist2(x1: Int, y1: Int, x2: Int, y2: Int) -> Int = (x1 - x2) ** 2 + (y1 - y2) ** 2

@!multitest
let n = read<1..=1e5>()
let circles = read<[n](x: 1..=1e9, y: 1..=1e9)>()
let (xs, ys, xt, yt): (1..=1e9, 1..=1e9, 1..=1e9, 1..=1e9) = read()
let mine = dist2(xs, ys, xt, yt)
print(yes(circles.all(c => dist2(c.x, c.y, xt, yt) > mine)))
