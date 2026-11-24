// Source: AtCoder Educational DP Contest W (Intervals), https://atcoder.jp/contests/dp/tasks/dp_w
// Problem: a 0/1 string of length n <= 2e5 and m <= 2e5 rules (l <= r, a), |a| <= 1e9: a rule
//   adds a to the score if the string has a 1 in l..=r. Print the largest possible score.
// Solution: best[i] = the best score with the last 1 at i (0: none), counting the rules with
//   r <= i. A SegTree with range add and max over i: set best[i] = max best[0..i), then each rule
//   ending at i adds a to best[l..=i], the states whose last 1 lies inside it. The answer is the
//   max over all states: O((n + m) log n).
// original: corpus/submissions/4133460.cpp
let (n, m): (1..=2e5, 1..=2e5) = read()
domain Pos = 1..=n
let rules: [m](l: Pos, r: Pos, a: -1e9..=1e9) = read()

var by_end: [Pos]List<(l: Pos, a: Int)> = [Pos] _ => []
for (l, r, a) in rules: by_end[r].push((l:, a:))

domain Last = 0..=n
var best = SegTree<Last, Int, Int>(max, fill: 0, act: add)
for i in Pos:
    let at = Last(i.label)
    best.set(at, best.fold(Last.first!..at))
    for (l, a) in by_end[i]: best.apply(Last(l.label)..=at, a)
print(best.fold(Last.first!..=Last.last!))
