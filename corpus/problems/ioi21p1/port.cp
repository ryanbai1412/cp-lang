// Source: IOI 2021 Day 1 Distributing Candies, https://dmoj.ca/problem/ioi21p1
// Problem: n <= 2e5 boxes with capacities c <= 1e9, all empty, and q <= 2e5 days l r v
//   (|v| <= 1e9, v != 0): every box in l..=r gets p -> min(c, p + v) or max(0, p + v). Print the
//   final counts.
// Solution: sweep the boxes, keeping a segment tree over days of each day's change to the current
//   box, with each node's end, highest and lowest prefix sums. For box capacity c, walk down for
//   the latest day p from which the walk spans at least c: from then on the walls bind once, so
//   the answer is c - (top - end) or end - low of the days after p by the sign of v[p], and
//   end - low of all days if no such p. O((n + q) log q).
// original: corpus/submissions/4698803.cpp (grader function distribute_candies)
// stdin: n, the n capacities, q, then q lines l r v. stdout: the final counts.
let n: 1..=200000 = read()
let cap: [0..n]Int = read()
let q: 1..=200000 = read()
let ops: [0..3 * q]Int = read()

// Sweep the boxes left to right keeping, per day, the change that day makes
// to the current box (0 when the box is outside the day's range). A stretch
// of days is summarised by the walk it makes from 0 without walls: where it
// ends, and the highest and lowest points it touches (the start included).
var end: List<Int> = []
var top: List<Int> = []
var low: List<Int> = []
for _ in 0..4 * q:
    end.push(0)
    top.push(0)
    low.push(0)

fn set(o: Int, l: Int, r: Int, day: Int, v: Int):
    if l == r:
        end[o] = v
        top[o] = max(v, 0)
        low[o] = min(v, 0)
        return
    let mid = (l + r) / 2
    if day <= mid: set(2 * o, l, mid, day, v)
    else: set(2 * o + 1, mid + 1, r, day, v)
    end[o] = end[2 * o] + end[2 * o + 1]
    top[o] = max(top[2 * o], end[2 * o] + top[2 * o + 1])
    low[o] = min(low[2 * o], end[2 * o] + low[2 * o + 1])

// The walk over the days after the node being visited.
var tail_end = 0
var tail_top = 0
var tail_low = 0
// Latest day p whose walk over days p.. spans at least c, or -1. The tail is
// left holding the days after p (all days when there is no such p).
fn last_wide(o: Int, l: Int, r: Int, c: Int) -> Int:
    let e = end[o] + tail_end
    let t = max(top[o], end[o] + tail_top)
    let b = min(low[o], end[o] + tail_low)
    if t - b < c:
        tail_end = e
        tail_top = t
        tail_low = b
        return -1
    if l == r: return l
    let mid = (l + r) / 2
    let right = last_wide(2 * o + 1, mid + 1, r, c)
    if right != -1: return right
    last_wide(2 * o, l, mid, c)

var starts: List<List<Int>> = []
var stops: List<List<Int>> = []
for _ in 0..=n:
    starts.push([])
    stops.push([])
for day in 0..q:
    starts[ops[3 * day]].push(day)
    stops[ops[3 * day + 1] + 1].push(day)

var ans: List<Int> = []
for box in 0..n:
    for day in starts[box]: set(1, 0, q - 1, day, ops[3 * day + 2])
    for day in stops[box]: set(1, 0, q - 1, day, 0)
    tail_end = 0
    tail_top = 0
    tail_low = 0
    let p = last_wide(1, 0, q - 1, cap[box])
    if p == -1:
        // never spans the box: only the floor matters
        ans.push(tail_end - tail_low)
    elif ops[3 * p + 2] > 0:
        // day p starts a climb of a full box: full at the tail's highest point
        ans.push(cap[box] - (tail_top - tail_end))
    else:
        // day p starts a drop of a full box: empty at the tail's lowest point
        ans.push(tail_end - tail_low)
print(ans)
