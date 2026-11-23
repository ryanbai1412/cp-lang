// IOI 2014 Holiday: cities stand in a line, each with a number of
// attractions. Starting in one city, each day is spent moving to a
// neighbouring city or visiting the city you are in (once). See the most
// attractions in d days.
// original: corpus/submissions/4691210.cpp (a grader function; batch 5 K20)
// This sketch was not run.
use std.dp.{dnc_opt}   // WISH R6
use std.seq.{TopSums}  // WISH: the k largest of a prefix, summed

let (n, start, days): (1..=1e5, 0..n, 0..=250000) = read()
domain City = 0..n
let fun: [City]0..=1e9 = read()

// most[t]: the most seen in t days spent on `side`, the cities in the order
// they are passed going away from the start. Reaching the i-th costs `step`
// days a city: 2 if the walk has to come back, 1 if it ends there.
fn away(side: List<Int>, step: Int) -> [0..=days]Int:
    let top = TopSums(side)
    // with more days it never pays to turn round sooner, so the best last city only moves away
    dnc_opt(0..=days, side.keys, (t, i) => top.sum(..=i, largest: t - (i + 1) * step))  // WISH R6

// walk `first` and come back, then walk `second`; the start city is visited or not
fn best(first: List<Int>, second: List<Int>) -> Int:
    let (round_trip, one_way) = (away(first, 2), away(second, 1))
    max(
        here + round_trip[t] + one_way[days - spent - t] for (spent, here) in [
            (0, 0),
            (1, fun[start]),
        ] for t in 0..=days - spent
    )

let (left, right) = (fun[..start].rev, fun[start.next..])  // WISH K10: .rev on a slice
print(max(best(right, left), best(left, right)))
