// IOI '09 Salesman (DMOJ: rpeng3)
// Ideal v2 port. original: corpus/submissions/4811952.cpp
//
// Fairs lie along a river: fair i is on day t, at position `at`, and earns
// `gain`. Moving one unit upstream costs `up`, downstream `down`. Starting and
// ending at `home`, visit fairs in day order for the largest profit.
//
// Sweep days in order. Two max segment trees over positions answer "best
// profit so far, arriving from below" and "from above". Fairs on the same day
// form a stretch that is entered anywhere and swept to one end and back.
const TOP = 500_001
let (n, up, down, home): (1..=5e5, 1..=10, 1..=10, 1..=TOP) = read()
domain Pos = 1..=TOP
let fairs: [0..n](t: 1..=5e5, at: Pos, gain: 1..=4000) = read()

var by_day = Map<Int, List<(at: Pos, gain: Int)>>()
for (t, at, gain) in fairs: (by_day[t] ??= []).push((at, gain))
(by_day[TOP] ??= []).push((home, 0))            // the trip ends back home

// from_low[p] = profit at p plus p * down, so a stop x above p subtracts
// x * down; from_high mirrors it. `?Int` with bare `max` needs no -inf.
var from_low = SegTree<Pos, ?Int>(max)
var from_high = SegTree<Pos, ?Int>(max)
fn leave(p: Pos, profit: Int):
    from_low.set(p, profit + p * down)
    from_high.set(p, profit - p * up)
leave(home, 0)

// WISH X1: this wants to be
//     max(from_low.fold(..=x) - x * down, from_high.fold(x..) + x * up)
// but an optional cannot be an operand of `-` or `+`.
fn arrive(x: Pos) -> Int:
    var best: ?Int = none
    if let v = from_low.fold(..=x): best max= v - x * down
    if let v = from_high.fold(x..): best max= v + x * up
    best!

for (_, day) in by_day.items:
    let fs = sorted(day)
    let m = fs.len
    let pre = prefix(f.gain for f in fs)
    let entry = [arrive(f.at) for f in fs]
    var opt: [0..m]?Int = none
    // Enter at i, carry on to a fair at a larger position, turn, and leave at j <= i.
    var (far, back): (?Int, ?Int) = (none, none)
    for i in (0..m).rev:
        let x = fs[i].at
        far max= pre.sum(..=i) - (up + down) * x
        back max= far! + down * x + entry[i]
        opt[i] max= back! + up * x - pre.sum(..i)
    // Enter at i, carry on to a fair at a smaller position, turn, and leave at j >= i.
    (far, back) = (none, none)
    for i in 0..m:
        let x = fs[i].at
        far max= (up + down) * x - pre.sum(..i)
        back max= far! - up * x + entry[i]
        opt[i] max= back! - down * x + pre.sum(..=i)
    for (i, f) in fs.items: leave(f.at, opt[i]!)
print(arrive(home))
