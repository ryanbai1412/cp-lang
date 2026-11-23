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
domain Pos = 0..n
let a: [Pos]1..=1e9 = read()
let ks: [q]1..=1e9 = read()
let window = prefix(a)
let cuts = compress(a + [2 * x for x in a])
let cap = ks.max() + a.max()
let by_k = (0..q).sorted(by: i => ks[i])

fn raise(l: 1..=31, x: Int) -> Int:
    let y = x - 1
    l + 2 * y - (y >> (l - 1)) - popcount(y & ((1 << (l - 1)) - 1))

// Convert a compressed cut run `r` into its `(a[s] * 2^(l - 1), a[s - 1] * 2^l]` X-range.
fn x_range(r: Range<cuts.D>, l: 1..=31) -> Range<Int>:
    let lo = (cuts[r.first!] << (l - 1)) + 1
    let hi = if let nxt = r.last!.next: min(cap, cuts[nxt] << (l - 1)) else: cap
    lo..=hi

var ans: [q]Int = a.max()
for l in 1..=31:
    var saved = IntervalMap<cuts.D, ?Int>.new(None)
    let windows = [(s, window.sum(s..=e)) for s in Pos if let e = s + (l - 1)].sorted(by: w => w.1)
    for (s, total) in windows:
        let end = if let p = s.prev: cuts.index_of(2 * a[p]).l else: cuts.D.last.r
        saved.assign(cuts.index_of(a[s]).l..end, total)
    var waiting = by_k.clone()
    for (r, opt) in saved.each().rev:
        if let best = opt:
            let xr = x_range(r, l)
            if not xr.is_empty:
                let floor = raise(l, xr.first!)
                while let x = waiting.last, ks[x] + best >= floor:
                    ans[x] max= xr.last_true(h => raise(l, h) <= ks[x] + best)!
                    waiting.pop()
print(ans)
