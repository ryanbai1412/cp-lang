// Batch #12: `xs.last(k)` is the element `k` back from the last one.
let n: Int = read()
var xs: List<Int> = [read() for _ in 0..n]
print(xs.last(0) ?? -1, xs.last(1) ?? -1, xs.last(n - 1) ?? -1, xs.last(n) ?? -1, xs.last(-1) ?? -1)
var hull: List<(x: Int, y: Int)> = []
for i in 0..n:
    while hull.len >= 2 and hull.last(1)!.y >= xs[i]: hull.pop()
    hull.push((i, xs[i]))
print(hull.len, hull.last(1)!.x, [x * 2 for x in xs].last(2) ?? -1)
if let prev = xs.last(1): print(prev + 1)
