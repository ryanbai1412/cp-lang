// AtCoder DP Z Frog 3: jumping from stone j to i costs (h[i] - h[j])^2 + c;
// heights increase. cost[i] = h[i]^2 + c + min over j of the line
// (-2 h[j]) x + cost[j] + h[j]^2 at x = h[i]. Slopes fall and queries rise,
// so the lower hull is a queue.
use std.geometry.{dot, turn}

let (n, c): (2..=2e5, 1..=1e12) = read()
let h: [0..n]1..=1e6 = read()

var hull = Deque<Pt>()
var cost = 0
for x in h:
    let q = Pt(x, 1)
    while hull.len >= 2 and dot(hull[1], q) <= dot(hull[0], q): hull.pop_front()
    if let best = hull.first: cost = dot(best, q) + x * x + c
    let line = Pt(-2 * x, cost + x * x)
    while hull.len >= 2 and turn(hull[hull.len - 2], hull.last!, line) >= 0: hull.pop()
    hull.push(line)
print(cost)
