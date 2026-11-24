// Source: IOI 2014 P6 (Holiday, standard I/O), https://dmoj.ca/problem/ioi14p6io
// limits: 1800 ms, 256 MB
// Problem: n <= 1e5 cities on a line with attraction counts <= 1e9, a start city and
//   d <= 2n + n/2 days; each day moves to a neighbour or visits the current city (each city
//   counts once). Print the most attractions visitable.
// Solution: a best walk goes out one way and back, then out the other way. For one side and t
//   days, ending at the i-th city passed costs step days per city (2 if coming back, 1 if not)
//   and visits the best of the first i with the rest; the best i only moves out as t grows, so
//   monotone_minima finds it, with WaveletMatrix top-k sums: O((n + d) log^2 n). Combine both
//   sides over every split of the days, visiting the start city or not.
// original: corpus/submissions/4691224.cpp
// Bounds: n: 1..=1e5; start: 0..=n - 1; days: 0..=250000; fun: 0..=1e9
use std.segtree.{WaveletMatrix}

let (n, start, days): (1..=1e5, 0..n, 0..=250000) = read()
domain City = 0..n
let fun: [City]0..=1e9 = read()
let home = City.at(start)!

// away(side): (round trip, one way) for one side, the cities in the order they
// are passed going away from the start, both from one WaveletMatrix. most[t] is
// the most seen in t days on the side: reaching the i-th costs `step` days a
// city, 2 if the walk has to come back and 1 if it ends there, and the days left
// go to the best cities passed.
fn away(side: List<Int>) -> (List<Int>, List<Int>):
    let order = WaveletMatrix(side)
    let passed = Prefix<Int>(side)
    fn most(step: Int) -> List<Int>:
        fn seen(t: Int, i: Int) -> ?Int:
            let visits = min(t - (i + 1) * step, i + 1)
            if visits < 0: return None
            Some(-(passed.cut(i + 1) - order.quantile_sum(0..i + 1, i + 1 - visits)))
        // with more days it never pays to turn round sooner, so the best last city only moves away
        [if let b = best: -b.val else: 0 for best in monotone_minima(days + 1, side.len, seen)]
    (most(2), most(1))

// walk one side and come back, then walk the other; the start city is visited or not
fn best(round_trip: List<Int>, one_way: List<Int>) -> Int:
    [
        here + round_trip[t] + one_way[days - spent - t]
        for (spent, here) in [(0, 0), (1, fun[home])]
        for t in 0..=days - spent
    ].max()!

let (left_back, left_on) = away(fun[..home].rev.collect())
let (right_back, right_on) = away(fun[home..].drop(1).collect())
print(max(best(right_back, left_on), best(left_back, right_on)))
