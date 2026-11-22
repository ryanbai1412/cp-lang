// Codeforces 2057 F: the tallest pupil after at most k pizzas, the line staying comfortable (2·a_i >= a_{i+1}).
// Raising pupil i to X lifts the l pupils s = i−l+1 ..= i to ⌈X / 2^j⌉; l is the right count exactly when
// the first of them is below its target and the pupil before already meets it: X ∈ (a_s·2^(l−1), a_{s−1}·2^l].
// The cost Σ_{j<l} ⌈X / 2^j⌉ − Σ a[s..=i] rises with X, so for each l <= 31 keep the window with the largest sum
// on every stretch of X; sweeping the stretches from high to low answers the queries in decreasing k.
@!multitest
let (n, q): (1..=50_000, 1..=50_000) = read()
let a: [n]1..=1e9 = read()
let ks: [q]1..=1e9 = read()
let window = prefix(a)
let cuts = compress(a + [2 * x for x in a])
let cap = ks.max() + a.max()
let by_k = (0..q).sorted(by: i => ks[i])

// Σ_{j<l} ⌈x / 2^j⌉ = l + 2y − ⌊y / 2^(l−1)⌋ − popcount(y mod 2^(l−1)) with y = x − 1
fn raise(l: Int, x: Int) -> Int:
    let y = x - 1
    l + 2 * y - (y >> (l - 1)) - popcount(y & ((1 << (l - 1)) - 1))

var ans: [q]Int = a.max()
for l in 1..=31:
    var saved = IntervalMap<cuts.D, ?Int>.new(none)
    for s in (0..=n - l).sorted(by: s => window.sum(s..s + l)):
        let end = if s > 0: cuts.at(2 * a[s - 1]).l else: cuts.D.last.r
        saved.assign(cuts.at(a[s]).l..end, window.sum(s..s + l))
    var waiting = by_k
    for (r, best) in [run for run in saved.each()].reversed():
        if let best = best:
            let (_, j) = r.ends
            let lo = (cuts[r.first!] << (l - 1)) + 1
            let hi = if j < cuts.len: min(cap, cuts[cuts.D.nth(j)] << (l - 1)) else: cap
            if lo <= hi:
                let floor = raise(l, lo)
                while let x = waiting.last, ks[x] + best >= floor:
                    ans[x] max= bsearch_last(lo..=hi, h => raise(l, h) <= ks[x] + best)!
                    waiting.pop()
print(ans)
