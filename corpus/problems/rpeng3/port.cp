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
var fairs: [Fair](t: 1..=500_000, at: Loc, gain: 1..=4000) = read()

// Every visited location's best profit lies in -5e6..=2e9: walking there from
// home costs at most 10 * 5e5, and the fairs gain at most 4000 * 5e5. The keys
// below move it by at most 5e6 either way, so Key's low end is below any of
// them and is the trees' empty fold.
type Key = -10_000_000..=2_010_000_000

// from_low[p] = best profit ending at p, plus p * down: a later stop x >= p
// subtracts x * down. from_high mirrors it for stops x <= p.
var from_low = SegTree<Loc, Key>(max)
var from_high = SegTree<Loc, Key>(max)
from_low.set(home, Int(home) * down)
from_high.set(home, -Int(home) * up)

fn arrive(x: Loc) -> Int:
    max(from_low.fold(Loc.first!..=x) - Int(x) * down, from_high.fold(x..=Loc.last!) + Int(x) * up)

let ud = up + down
fairs.sort()
for day in fairs.runs(by: f => f.t):
    // A day's fairs, by location: enter at one, sweep to an end of a
    // contiguous stretch and come back to the exit.
    domain Q = 0..day.len
    let stop = [Q] q => day.nth(q.ord)
    let x = [Q] q => Int(stop[q].at)
    let sums = ([Q] q => stop[q].gain).scan(+)
    let before = [Q] q => sums.prefix(q.l)
    let upto = [Q] q => sums.prefix(q.r)
    let entry = [Q] q => arrive(stop[q].at)
    // Enter at q, go up to the far end, come back down to an exit at or below q.
    let far_up = ([Q] q => upto[q] - ud * x[q]).rev.scan(max)
    let enter_up = [Q] q => far_up.suffix(q.l) + ud * x[q] - before[q] + entry[q]
    let exit_up = ([Q] q => before[q] - up * x[q] + enter_up[q]).rev.scan(max)
    // Enter at q, go down to the near end, come back up to an exit at or above q.
    let far_down = ([Q] q => ud * x[q] - before[q]).scan(max)
    let enter_down = [Q] q => far_down.prefix(q.r) - ud * x[q] + upto[q] + entry[q]
    let exit_down = ([Q] q => enter_down[q] + down * x[q] - upto[q]).scan(max)
    for q in Q:
        let best = max(
            exit_up.suffix(q.l) + up * x[q] - before[q],
            exit_down.prefix(q.r) - down * x[q] + upto[q],
        )
        from_low.set(stop[q].at, best + x[q] * down)
        from_high.set(stop[q].at, best - x[q] * up)
print(arrive(home))
