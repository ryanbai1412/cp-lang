// Codeforces 2057 F: Formation.
// Tallest pupil after at most `k` pizzas while keeping the line comfortable
// (`2 * a[i] >= a[i + 1]`).
// Raising pupil `i` to `X` lifts the `l` pupils `s = i - l + 1 ..= i` to
// `ceil(X / 2^j)`; `l` is the exact affected window iff `a[s]` needs lifting
// and `a[s - 1]` already satisfies the constraint: `X in (a[s] * 2^(l - 1), a[s - 1] * 2^l]`.
// With `y = X - 1`, `ceil(X / 2^j) = 1 + floor(y / 2^j)`. Summing `j = 0..l`
// using the binary-digit identity `sum_{j < l} floor(y / 2^j) = 2y - floor(y / 2^(l - 1)) - popcount(y mod 2^(l - 1))`
// gives the closed form `raise(l, X)`.
// Since `raise(l, X) - sum(a[s..=i])` is increasing in `X`, for each `l <= 31`
// keep the maximum window sum on each `X`-interval and sweep from high to low
// against queries sorted by `k`.

@!multitest
use std.interval.IntervalMap

let (n, q): (1..=50_000, 1..=50_000) = read()
let a: [n]1..=1e9 = read()
let ks: [q]1..=1e9 = read()
let window = prefix(a)
let cuts = compress(a + [2 * x for x in a])
let cap = ks.max() + a.max()
let by_k = (0..q).sorted(by: i => ks[i])

fn raise(l: 1..=31, x: Int) -> Int:
    let y = x - 1
    l + 2 * y - (y >> (l - 1)) - popcount(y & ((1 << (l - 1)) - 1))

var ans: [q]Int = a.max()
for l in 1..=31:
    var saved = IntervalMap<cuts.D, ?Int>.new(none)
    for s in (0..=n - l).sorted(by: s => window.sum(s..s + l)):
        let end = if s > 0: cuts.at(2 * a[s - 1]).l else: cuts.D.last.r
        saved.assign(cuts.at(a[s]).l..end, window.sum(s..s + l))
    var waiting = by_k
    for (r, opt) in saved.runs.rev:
        if let best = opt:
            let (_, j) = r.ends
            let lo = (cuts[r.first!] << (l - 1)) + 1
            let hi = if j < cuts.len: min(cap, cuts[cuts.D.nth(j)] << (l - 1)) else: cap
            if lo <= hi:
                let floor = raise(l, lo)
                while let x = waiting.last, ks[x] + best >= floor:
                    ans[x] max= bsearch_last(lo..=hi, h => raise(l, h) <= ks[x] + best)!
                    waiting.pop()
print(ans)
