// AtCoder DP Z Frog 3: jumping from stone j to i costs (h[i] - h[j])^2 + c;
// heights increase. cost[i] = h[i]^2 + c + min over j of the line
// (-2 h[j]) x + cost[j] + h[j]^2 at x = h[i]. Slopes fall and queries rise,
// so the lower hull is a queue.
let (n, c): (2..=2e5, 1..=1e12) = read()
let h: [0..n]1..=1e6 = read()

type Line = (k: Int, b: Int)
fn at(l: Line, x: Int) -> Int = l.k * x + l.b
// with slopes a.k > b.k > c.k, b is never the lowest
fn useless(a: Line, b: Line, c: Line) -> Bool =
    (c.b - a.b) * (a.k - b.k) <= (b.b - a.b) * (a.k - c.k)

var hull = Deque<Line>()
var cost = 0
for x in h:
    while hull.len >= 2 and at(hull.get(1), x) <= at(hull.get(0), x): hull.pop_front()
    if let best = hull.first: cost = at(best, x) + x * x + c
    let line: Line = (-2 * x, cost + x * x)
    while hull.len >= 2 and useless(hull.get(hull.len - 2), hull.last!, line): hull.pop()
    hull.push(line)
print(cost)
