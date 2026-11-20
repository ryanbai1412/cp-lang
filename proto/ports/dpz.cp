// cp: 0.1 1f59ac0+dirty
// AtCoder DP Z Frog 3: jumping from stone j to i costs (h[i] - h[j])^2 + c;
// heights increase. cost[i] = h[i]^2 + c + min over j of the line
// (-2 h[j]) x + cost[j] + h[j]^2 at x = h[i]. Slopes fall and queries rise,
// so the lower hull is a queue.
let n: 2..=2e5 = read()
let c: 1..=1e12 = read()
domain I = 1..=n
fn rising() -> [I]1..=1e6:
    var h: [I]1..=1e6 = 1
    for i in I:
        if let p = i.prev: h[i] = h[p] + rand(1, 4)
    h
let h: [I]1..=1e6 = read(gen: rising) where all(h[i.prev!] < h[i] for i in I if i != I.first)

type Line = (k: Int, b: Int)
fn at(l: Line, x: Int) -> Int: l.k * x + l.b
// with slopes a.k > b.k > c.k, b is never the lowest
fn useless(a: Line, b: Line, c: Line) -> Bool:
    (c.b - a.b) * (a.k - b.k) <= (b.b - a.b) * (a.k - c.k)

var hull: List<Line> = []
var front = 0
var cost = 0
for i in I:
    if i != I.first:
        while front + 1 < hull.len and at(hull[front + 1], h[i]) <= at(hull[front], h[i]): front += 1
        cost = at(hull[front], h[i]) + h[i] * h[i] + c
    let line: Line = (-2 * h[i], cost + h[i] * h[i])
    while hull.len >= front + 2 and useless(hull[hull.len - 2], hull[hull.len - 1], line): hull.pop()
    hull.push(line)
print(cost)
