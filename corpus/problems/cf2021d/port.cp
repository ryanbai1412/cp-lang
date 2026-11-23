// Source: Codeforces 2021D (Boss, Thirsty), https://codeforces.com/contest/2021/problem/D
// Problem: n days, m >= 3 drinks, profit A[i][j] (|A| <= 1e9, sum n * m <= 2e5). Each day
//   sells a subarray of drinks; from day 2 on it must share a drink with the day before and
//   also have one that day lacked. Print the max total profit. Multitest.
// Solution: a day is a pair of cuts l < r; the next day is valid iff it strictly contains an
//   endpoint p of this one. end[p] = best total so far with the last day ending at cut p. A
//   new right end r earns pref[r] + max over p < r of open[p], where open[p] = end[p] - min
//   over l < p of pref[l] (day one: l = p, so -pref[p]). One sweep keeps both running
//   extremes; left ends are the same sweep from the right with pref negated. Every cut is
//   reached from one side or the other because m >= 3. O(nm).
@!multitest
let (n, m): (1..=200_000, 3..=200_000) = read()
domain Drink = 0..m
type Cut = Drink.cuts

// best[r] max= the best day whose far end r is reached last in `order`.
fn sweep<S>(mut best: [Cut]?Int, order: S, pref: Cut -> Int, end: ?[Cut]Int):
    var open: ?Int = None  // best open[p], p before here
    var low: ?Int = None   // least pref[l], l before here
    for c in order:
        if let o = open: best[c] max= pref(c) + o
        // `end!`, not `if let e = end`: binding the array copies it every cut (friction 1).
        open max= if end.is_some: low.map(l => end![c] - l) else: Some(-pref(c))
        low min= pref(c)

var end: ?[Cut]Int = None
repeat n:
    let pre = read<[Drink]-1e9..=1e9>().scan(+)
    var next: [Cut]?Int = None
    sweep(mut next, Drink.cuts, c => pre[c], end)
    sweep(mut next, Drink.cuts.rev, c => -pre[c], end)
    end = [Cut] p => next[p]!
print(end!.max())
