// Source: IOI 2009 Day 2 (Salesman), https://dmoj.ca/problem/rpeng3
//   (not public on DMOJ; statement from the official IOI 2009 task)
// limits: 3000 ms, 64 MB
// Problem: N <= 5e5 fairs, each a day 1 <= T <= 5e5, a location 1 <= L <= 500001 (distinct, never
//   home) and a gain 1 <= M <= 4000. A metre upstream (towards smaller L) costs U, downstream D,
//   1 <= D <= U <= 10. Start and end at home 1 <= S <= 500001; fairs go in day order, same-day
//   fairs in any order. Print the largest total gain minus travel cost.
// Solution: best[p] = profit standing at location p. Reaching x from p <= x costs (x - p) D
//   and from p >= x costs (p - x) U, so keep best[p] + p D and best[p] - p U in two max
//   SegTrees and fold each side of x. A day's fairs, sorted by location, are a contiguous
//   stretch: enter at one, sweep to an end and back to the exit; two prefix and two suffix
//   running-max passes over prefix gains give every exit. The trip ends by arriving home.
//   O(N log L).
// Bounds: n, fairs.t: 1..=500_000; up: 1..=10; down: 1..=up; home, fairs.at: 1..=500_001;
//   fairs.gain: 1..=4000
let n: 1..=500_000 = read()
let up: 1..=10 = read()
let down: 1..=up = read()
domain Loc = 1..=500_001
let home: Loc = read()
domain Fair = 0..n
let fairs: [Fair](t: 1..=500_000, at: Loc, gain: 1..=4000) = read()
let neg = -1000000000000000000

// from_low[p] = best profit ending at p, plus p * down: a later stop x >= p
// subtracts x * down. from_high mirrors it for stops x <= p.
var from_low = SegTree<Loc, Int>(max, fill: neg)
var from_high = SegTree<Loc, Int>(max, fill: neg)
from_low.set(home, Int(home) * down)
from_high.set(home, -Int(home) * up)

fn arrive(x: Loc) -> Int:
    max(from_low.fold(Loc.first!..=x) - Int(x) * down, from_high.fold(x..=Loc.last!) + Int(x) * up)

let by_day = fairs.sorted()
for day in by_day.runs(by: f => f.t):
    // A day's fairs: visit a contiguous stretch, entering anywhere,
    // sweeping to one end and optionally back.
    let m = day.len
    let locs = [f.at for f in day]
    let at = [Int(x) for x in locs]
    var pre = [0]
    for f in day: pre.push(pre.last! + f.gain)
    let entry = [arrive(x) for x in locs]
    var opt = List.filled(m, neg)
    var val = List.filled(m, neg)
    // Enter at q, go up to the far end, come back down to the exit.
    var run = neg
    for q in (0..m).rev:
        run max= -(up + down) * at[q] + pre[q + 1]
        val[q] = run + (up + down) * at[q] - pre[q] + entry[q]
    run = neg
    for q in (0..m).rev:
        run max= -up * at[q] + pre[q] + val[q]
        opt[q] max= run + up * at[q] - pre[q]
    // Enter at q, go down to the near end, come back up to the exit.
    run = neg
    for q in 0..m:
        run max= (up + down) * at[q] - pre[q]
        val[q] = run - (up + down) * at[q] + pre[q + 1] + entry[q]
    run = neg
    for q in 0..m:
        run max= down * at[q] - pre[q + 1] + val[q]
        opt[q] max= run - down * at[q] + pre[q + 1]
    for q in 0..m:
        from_low.set(locs[q], opt[q] + at[q] * down)
        from_high.set(locs[q], opt[q] - at[q] * up)
print(arrive(home))
