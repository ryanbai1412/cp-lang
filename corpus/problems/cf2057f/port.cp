// Source: Codeforces 2057F (Formation), https://codeforces.com/contest/2057/problem/F
// Problem: heights a[1..n] (<= 1e9) with 2a[i] >= a[i+1]; a pizza adds 1 to one pupil. For each of
//   q queries k (<= 1e9) print the largest max(a) reachable with at most k pizzas while keeping
//   2a[i] >= a[i+1]. n, q <= 5e4 (sums over all tests). Multitest.
// Solution: raising pupil i to X forces pupil i - j up to ceil(X / 2^j), so only a window s..=i of
//   l <= 31 pupils moves; it is exactly that window iff X / 2^(l-1) is in (a[s], 2a[s-1]], at cost
//   raise(l, X) - sum a[s..=i], raise in closed form. Per l, an IntervalMap over compressed
//   X / 2^(l-1) keeps the largest window sum per range (assigned in ascending sum); ranges high to
//   low meet queries by descending k, binary-searching X in the first range that fits.
//   O(31 (n + q) log).

@!multitest
use std.interval.IntervalMap

let (n, q): (1..=50_000, 1..=50_000) = read()
domain Pos = 0..n
let a: [Pos]1..=1e9 = read()
let ks: [q]1..=1e9 = read()
let window = prefix(a)
let (C, cuts) = compress(a + [2 * x for x in a])
let cap = ks.max()! + a.max()!
let by_k = (0..q).sorted(by: i => ks[i])

fn raise(l: 1..=31, x: Int) -> Int:
    let y = x - 1
    l + 2 * y - (y >> (l - 1)) - popcount(y & ((1 << (l - 1)) - 1))

var ans: [q]Int = a.max()!
for l in 1..=31:
    var saved = IntervalMap<C, ?Int>.new(None)
    let windows = [
        (s, window.fold(s..=e)) for s in Pos if let e = s.offset(l - 1)
    ].sorted(by: w => w.1)
    for (s, total) in windows:
        let end = if let p = s.prev: cuts.at(2 * a[p]).l else: C.last!.r
        saved.assign(cuts.at(a[s]).l..end, Some(total))
    var waiting = by_k.clone()
    for (r, opt) in saved.each().collect().rev:
        if let best = opt:
            let lo = (cuts[r.first!] << (l - 1)) + 1
            let hi = if let nxt = r.last!.next: min(cap, cuts[nxt] << (l - 1)) else: cap
            let xr = lo..=hi
            if not xr.is_empty:
                let floor = raise(l, xr.first!)
                while let x = waiting.last, ks[x] + best >= floor:
                    ans[x] max= xr.last_true(h => raise(l, h) <= ks[x] + best)!
                    waiting.pop()
print(ans)
