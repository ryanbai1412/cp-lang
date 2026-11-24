// Codeforces 2021 D, tuned: port.cp with the running extremes and best ends as plain Int
// with a far-below sentinel, as orig.cpp's -INF, instead of ?Int with an empty case.
//
// Source: Codeforces 2021D (Boss, Thirsty), https://codeforces.com/contest/2021/problem/D
// limits: 2000 ms, 250 MB
// Problem: n days, m >= 3 drinks, profit A[i][j] (|A| <= 1e9, sum n * m <= 2e5). Each day
//   sells a subarray of drinks; from day 2 on it must share a drink with the day before and
//   also have one that day lacked. Print the max total profit. Multitest.
// Solution: a day is a pair of cuts l < r; the next day is valid iff it strictly contains an
//   endpoint p of this one. end[p] = best total so far with the last day ending at cut p. A
//   new right end r earns pref[r] + max over p < r of open[p], where open[p] = end[p] - min
//   over l < p of pref[l] (day one: l = p, so -pref[p]). One sweep keeps both running
//   extremes; left ends are the same sweep from the right with pref negated. Every cut is
//   reached from one side or the other because m >= 3. O(nm).
// Bounds: n: 1..=200_000; m: 3..=200_000
@!multitest
let (n, m): (1..=200_000, 3..=200_000) = read()
domain Drink = 0..m
type Cut = Drink.cuts
let far = 1_000_000_000_000_000_000

// best[r] max= the best day whose far end r is reached last in `order`; `far` stands for none.
fn sweep<S>(mut best: [Cut]Int, order: S, pref: Cut -> Int, end: [Cut]Int, first: Bool):
    var open = -far  // best open[p], p before here
    var low = far    // least pref[l], l before here
    for c in order:
        best[c] max= pref(c) + open
        open max= if first: -pref(c) else: end[c] - low
        low min= pref(c)

var end: [Cut]Int = 0
for day in 0..n:
    let pre = read<[Drink]-1e9..=1e9>().scan(+)
    var next: [Cut]Int = -far
    sweep(mut next, Drink.cuts, c => pre[c], end, day == 0)
    sweep(mut next, Drink.cuts.rev, c => -pre[c], end, day == 0)
    end = next
print(end.max()!)
