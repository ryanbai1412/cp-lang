// Source: COI 2007 #1 (Glasnici), https://dmoj.ca/problem/coi07p1
// limits: 600 ms, 32 MB
// Problem: n <= 1e5 messengers at sorted positions D <= 1e9 on a line; each runs at speed
//   <= 1 and hears a shouting messenger within distance K (real, <= 1e6). Only the first
//   knows the news at time 0. Print the least time until all know it, +-0.001.
// Solution: all know by time t iff everyone can move at most t so that consecutive
//   messengers end at most K apart. Greedy left to right with p, the furthest the previous
//   one can end: go to p + K if reachable within t, else to x + t if that is still within
//   p + K; otherwise t is too small, and raising it by half the remaining gap (everyone
//   before shifts right as much, this one comes left as much) closes it. O(n).
// original: corpus/submissions/5323995.cpp
// Bounds: n: 1..=1e5
let (k, n): (F64, 1..=1e5) = read()
let xs: [0..n]F64 = read()

var t = 0.0
var p = xs[0] - k
for x in xs:
    if abs(p + k - x) <= t: p += k
    elif x <= p + k: p = x + t
    else:
        let more = (x - t - p - k) / 2
        p = x - t - more
        t += more
print("{t:.3f}")
