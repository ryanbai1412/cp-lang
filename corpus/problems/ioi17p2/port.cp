// Source: IOI 2017 Day 1 P2 (Wiring), https://dmoj.ca/problem/ioi17p2
// Problem: n red and m blue points on a line (n, m <= 1e5, sorted, positions <= 1e9). Wire
//   points so that each has a wire to a point of the other colour; print the least total length.
// Solution: merge the points by position. cost[p] = cheapest wiring of the first p points: point
//   p - 1 either takes its own wire to the nearest other colour (near[], two scans), or closes
//   the block since the latest prefix with the same red-minus-blue balance, matched like
//   brackets at cost |sum of signed positions| over the block. last[] maps each balance to its
//   latest prefix, so the DP is O(n + m).
// original: corpus/submissions/5724153.cpp (grader function min_total_length)
// stdin: n m, the n red positions, the m blue positions (each sorted, all distinct).
// Bounds: n, m: 1..=100_000; red, blue: 0..=1_000_000_000 - 1
let n: 1..=100_000 = read()
let m: 1..=100_000 = read()
let red: [n]0..1_000_000_000 = read()
let blue: [m]0..1_000_000_000 = read()

// every point by position, sign +1 for red and -1 for blue
var points = [(at: Int(x), sign: 1) for x in red] + [(at: Int(x), sign: -1) for x in blue]
points.sort()
domain P = 0..n + m
fn pt(p: P) -> (at: Int, sign: Int) = points[Int(p)]

// near[p]: the distance from point p to the closest point of the other colour
const FAR = 1_000_000_000   // past every gap
var near: [P]0..=FAR = FAR  // to the nearest other colour; both occur
var (seen_red, seen_blue): (?Int, ?Int) = (None, None)
for p in P:
    if let x = if pt(p).sign == 1: seen_blue else: seen_red: near[p] min= pt(p).at - x
    if pt(p).sign == 1: seen_red = pt(p).at else: seen_blue = pt(p).at
(seen_red, seen_blue) = (None, None)
for p in P.rev:
    if let x = if pt(p).sign == 1: seen_blue else: seen_red: near[p] min= x - pt(p).at
    if pt(p).sign == 1: seen_red = pt(p).at else: seen_blue = pt(p).at

// cost[c]: the cheapest wiring of the points before cut c. Its last point takes its own wire
// to the nearest other colour, or the block since the latest cut with the same red-minus-blue
// balance is matched like brackets, at exactly |sum of signed positions| over the block.
domain Balance = -(n + m)..=n + m
var cost: [P.cuts]Int = 0
var signed: [P.cuts]Int = 0
var latest: [Balance]?Cut<P> = None
latest[Balance(0)] = P.cuts.first!
var bal = 0
for p in P:
    bal += pt(p).sign
    signed[p.r] = signed[p.l] + pt(p).sign * pt(p).at
    cost[p.r] = cost[p.l] + near[p]
    if let c = latest[Balance(bal)]: cost[p.r] min= cost[c] + abs(signed[p.r] - signed[c])
    latest[Balance(bal)] = p.r
print(cost[P.cuts.last!])
