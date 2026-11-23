// Source: IOI 2009 Day 2 (Salesman), https://dmoj.ca/problem/rpeng3
//   (not public on DMOJ; statement from the official IOI 2009 task)
// Problem: N <= 5e5 fairs, each a day T <= 5e5, a distinct location L <= 500001 and a gain
//   M <= 4000. A metre upstream (towards smaller L) costs U, downstream D, 1 <= D <= U <= 10.
//   Start and end at home S; fairs go in day order, same-day fairs in any order. Print the
//   largest total gain minus travel cost.
// Solution: best[p] = profit standing at location p. Reaching x from p <= x costs (x - p) D
//   and from p >= x costs (p - x) U, so keep best[p] + p D and best[p] - p U and take the max
//   on each side of x. A day's fairs, sorted by location, are a contiguous stretch: enter at
//   one, sweep to an end and back to the exit; two prefix and two suffix running-max passes
//   over prefix gains give every exit. A fair at home on day 2^19 ends the trip. This port
//   scans the two arrays for each arrival, O(N L); the C++ uses max segment trees, O(N log L).
let (n, up, down, home): (Int, Int, Int, Int) = read()
let neg = -1000000000000000000

var fairs: List<(t: Int, at: Int, gain: Int)> = []
var top = home
for _ in 0..n:
    let (t, at, gain): (Int, Int, Int) = read()
    fairs.push((t:, at:, gain:))
    top max= at
fairs.push((t: 1 << 19, at: home, gain: 0))  // the trip ends back home
fairs.sort()

// from_low[p] = best profit ending at p, plus p * down: a later stop x >= p
// subtracts x * down. from_high mirrors it for stops x <= p. The interpreter
// version scans these arrays; the ideal port keeps them in two max SegTrees.
var from_low = List.filled(top + 1, neg)
var from_high = List.filled(top + 1, neg)
from_low[home] = home * down
from_high[home] = -home * up

fn arrive(x: Int) -> Int:
    var best = neg
    for p in 0..x + 1: best max= from_low[p] - x * down
    for p in x..top + 1: best max= from_high[p] + x * up
    return best

var i = 0
while i < fairs.len:
    var j = i
    while j < fairs.len and fairs[j].t == fairs[i].t: j += 1
    // Fairs i..j share a day: visit a contiguous stretch, entering anywhere,
    // sweeping to one end and optionally back.
    let m = j - i
    var at: List<Int> = []
    var pre = [0]
    var entry: List<Int> = []
    for q in i..j:
        at.push(fairs[q].at)
        pre.push(pre.last! + fairs[q].gain)
        entry.push(arrive(fairs[q].at))
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
        from_low[at[q]] = opt[q] + at[q] * down
        from_high[at[q]] = opt[q] - at[q] * up
    i = j
print(from_low[home] - home * down)
