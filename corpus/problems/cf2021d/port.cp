// Source: Codeforces 2021D (Boss, Thirsty), https://codeforces.com/contest/2021/problem/D
// Problem: n days, m >= 3 drinks, profit A[i][j] (|A| <= 1e9, sum n * m <= 2e5). Each day
//   sells a subarray of drinks; from day 2 on it must share a drink with the day before and
//   also have one that day lacked. Print the max total profit. Multitest.
// Solution: a day is a pair of cuts l < r; the next day is valid iff it strictly contains an
//   endpoint p of this one. end[p] = best total so far with the last day ending at cut p. A
//   new right end r earns pref[r] + max over p < r of (end[p] - min over l < p of pref[l])
//   (day one: l = p, so -pref[p]); one sweep keeps both running extremes, and left ends are
//   the same sweep from the right with pref negated. O(nm).
@!multitest
let n: 1..=200_000 = read()
let m: 3..=200_000 = read()
domain Drink = 0..m
type Cut = Drink.cuts

fn sweep(order: List<Cut>, pref: Cut -> Int, opens: (Cut, ?Int) -> ?Int) -> [Cut]?Int:
    var best: [Cut]?Int = None
    var open: ?Int = None  // best opened day, p < here
    var low: ?Int = None   // least pref[l], l < here
    for c in order:
        if let o = open: best[c] = pref(c) + o
        open max= opens(c, low)
        low min= pref(c)
    best

var end: ?[Cut]Int = None
for _ in 0..n:
    let a: [Drink]-1_000_000_000..=1_000_000_000 = read()
    let pre = prefix(a)
    // A day opened at p: end[p] - pref[l] for some l < p, or l = p on day one.
    fn ends(order: List<Cut>, pref: Cut -> Int) -> [Cut]?Int:
        if let e = end: sweep(order, pref, (p, low) => if let l = low: e[p] - l else: None)
        else: sweep(order, pref, (p, _) => -pref(p))
    let right = ends(Drink.cuts, c => pre.prefix(c))
    let left = ends(Drink.cuts.rev, c => -pre.prefix(c))
    end = [Cut] p => max(right[p], left[p])!
print(end!.max())
