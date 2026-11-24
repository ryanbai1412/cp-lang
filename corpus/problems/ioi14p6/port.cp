// Source: IOI 2014 Day 2 Holiday, https://dmoj.ca/problem/ioi14p6
// Problem: n <= 1e5 cities on a line with attraction[i] <= 1e9 and a start city; each of d <=
//   2n + n/2 days either moves to a neighbouring city or visits the current one (a city counts
//   once). Print the most attractions visited.
// Solution: a best walk goes out one way and back, then out the other way. For one side and t
//   days, ending at the i-th city passed costs step days per city (2 if coming back, 1 if not)
//   and visits the best of the first i with the rest; the best i only moves out as t grows, so
//   monotone_minima finds it, with WaveletMatrix top-k sums: O((n + d) log^2 n). Combine both
//   sides over every split of the days, visiting the start city or not.
// original: corpus/submissions/4691210.cpp (a grader function)
// Bounds: n: 1..=1e5; start: 0..=n - 1; days: 0..=250000; fun: 0..=1e9
use std.segtree.{WaveletMatrix}

let (n, start, days): (1..=1e5, 0..n, 0..=250000) = read()
domain City = 0..n
let fun: [City]0..=1e9 = read()
let home = City.at(start)!

// most[t]: the most seen in t days spent on `side`, the cities in the order
// they are passed going away from the start. Reaching the i-th costs `step`
// days a city: 2 if the walk has to come back, 1 if it ends there; the days
// left go to the best cities passed.
fn away(side: List<Int>, step: Int) -> List<Int>:
    let order = WaveletMatrix(side)
    let passed = Prefix<Int>(side)
    fn seen(t: Int, i: Int) -> ?Int:
        let visits = min(t - (i + 1) * step, i + 1)
        if visits < 0: return None
        Some(-(passed.cut(i + 1) - order.quantile_sum(0..i + 1, i + 1 - visits)))
    // with more days it never pays to turn round sooner, so the best last city only moves away
    [if let b = best: -b.val else: 0 for best in monotone_minima(days + 1, side.len, seen)]

// walk `first` and come back, then walk `second`; the start city is visited or not
fn best(first: List<Int>, second: List<Int>) -> Int:
    let (round_trip, one_way) = (away(first, 2), away(second, 1))
    [here + round_trip[t] + one_way[days - spent - t] for (spent, here) in [
            (0, 0),
            (1, fun[home]),
        ] for t in 0..=days - spent].max()!

let left = [fun[c] for c in City if c < home].rev
let right = [fun[c] for c in City if c > home]
print(max(best(right, left), best(left, right)))
