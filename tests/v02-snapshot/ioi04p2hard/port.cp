// Source: IOI 2004 P2 (Hermes, hard), https://dmoj.ca/problem/ioi04p2hard
// Problem: from (0, 0), Hermes visits n <= 1e6 junctions (x_i, y_i) in order (|coords| <=
//   1e9), but serving one only needs reaching its row y = y_i or its column x = x_i. Print
//   the least total walking distance.
// Solution: between switches Hermes keeps one coordinate and walks the other from stop to
//   stop, so dx[i] / dy[i] = least cost of the rest when he holds the x / y of stop i - 1.
//   Going backwards, the next switch j costs a walk-prefix difference plus |held - coord_j|;
//   split the absolute value into below/above, and keep the best base -/+ coord per side in
//   monotone staircase maps keyed by coordinate. O(n log n).
// original: corpus/submissions/4182973.cpp
let n: Int = read()
type Stop = (x: Int, y: Int)
var stop: [0..=n]Stop = (0, 0)
for i in 1..=n: stop[i] = read()

var walk_x: [0..=n]Int = 0
var walk_y: [0..=n]Int = 0
for i in 1..=n:
    walk_x[i] = walk_x[i - 1] + abs(stop[i].x - stop[i - 1].x)
    walk_y[i] = walk_y[i - 1] + abs(stop[i].y - stop[i - 1].y)

// A staircase keeps only keys whose value beats every smaller key, so the
// minimum over keys <= x is the value at the last key <= x.
fn insert(mut stairs: Map<Int, Int>, k: Int, v: Int):
    if let (_, w) = stairs.last_le(k), w <= v: return
    while let (key, w) = stairs.first_gt(k), w > v: stairs.remove(key)
    stairs[k] = v

fn upto(stairs: Map<Int, Int>, x: Int) -> ?Int:
    let (_, v) = stairs.last_le(x) ?? return None
    v

var x_below = Map<Int, Int>()
var x_above = Map<Int, Int>()
var y_below = Map<Int, Int>()
var y_above = Map<Int, Int>()

fn offer_x(i: Int, cost: Int):
    if i == 0: return
    let base = walk_x[i - 1] + cost
    insert(mut x_below, stop[i].y, base - stop[i].y)
    insert(mut x_above, -stop[i].y, base + stop[i].y)

fn offer_y(i: Int, cost: Int):
    if i == 0: return
    let base = walk_y[i - 1] + cost
    insert(mut y_below, stop[i].x, base - stop[i].x)
    insert(mut y_above, -stop[i].x, base + stop[i].x)

var (dx, dy) = (0, 0)
offer_x(n, 0)
offer_y(n, 0)
for i in (0..n).rev:
    let prev = stop[if i > 0: i - 1 else: 0]
    dx = walk_y[n] - walk_y[i]
    dx min= upto(y_below, prev.x) + prev.x - walk_y[i]
    dx min= upto(y_above, -prev.x) - prev.x - walk_y[i]
    dy = walk_x[n] - walk_x[i]
    dy min= upto(x_below, prev.y) + prev.y - walk_x[i]
    dy min= upto(x_above, -prev.y) - prev.y - walk_x[i]
    offer_x(i, dx)
    offer_y(i, dy)

print(min(dx, dy))
