// Source: NOI 2019 P1 (Route), https://dmoj.ca/problem/noi19p1
// limits: 1000 ms, 512 MB
// Problem: m trains (x, y, p, q) leave station x at time p and reach y at q. Starting at station
//   1 at time 0, reach station n; each wait of t >= 0 (incl. before the first train) costs
//   A t^2 + B t + C, arriving at time z adds z; print the least total. n <= 1e5, m <= 2e5,
//   A <= 10, B, C <= 1e6, times <= 1000.
// Solution: dp[i] = cost to finish after riding train i. Sweep times downwards (departures
//   before arrivals at the same time, so q <= p is allowed): an arrival at e queries
//   min over later departures j of (-2A s_j) e + (A s_j^2 + B s_j + C + dp[j]), plus A e^2 - B e,
//   from a Li Chao tree per station over 0..=max time; a departure inserts its line. A
//   dummy train 1 -> 1 at time 0 holds the answer. O(m log T).
// original: corpus/submissions/3790564.cpp
let (n, m, a, b, c): (2..=1e5, 1..=2e5, 0..=10, 0..=1e6, 0..=1e6) = read()
domain Station = 1..=n
domain Train = 0..=m  // train m is the dummy 1 -> 1 at time 0
let given: [0..m](u: Station, v: Station, s: 0..=1000, e: 0..=1000) = read()
let trains = [Train] i => if i.ord < m: given[i.ord] else: (u: Station.first!, v: Station.first!, s: 0, e: 0)

var leaving = [Station] _ => LiChao(min, over: 0..=1000)
var after: [Train]?Int = None  // the least cost to finish once train i arrives
// latest first, departures before arrivals at the same time
let events = [(-trains[i].s, false, i) for i in Train] + [(-trains[i].e, true, i) for i in Train]
for (_, arrival, i) in events.sorted():
    let t = trains[i]
    if not arrival:
        if let cost = after[i]:
            leaving[t.u].add(Line(-2 * a * t.s, a * t.s * t.s + b * t.s + c + cost))
    elif t.v == Station.last:
        after[i] = t.e
    elif let wait = leaving[t.v].at(t.e):
        after[i] = wait + a * t.e * t.e - b * t.e
print(after[Train(m)]!)
