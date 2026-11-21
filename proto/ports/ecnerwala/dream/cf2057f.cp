// Codeforces 2057 F: the tallest pupil after at most k pizzas, the line staying comfortable (2·a_i >= a_{i+1}).
// Raising pupil i to X lifts the l pupils s = i−l+1 ..= i to ⌈X / 2^j⌉; l is the right count exactly when
// the first of them is below its target and the pupil before already meets it: X ∈ (a_s·2^(l−1), a_{s−1}·2^l].
// The cost Σ_{j<l} ⌈X / 2^j⌉ − Σ a[s..=i] rises with X, so for each l <= 31 keep the window with the largest sum on
// every stretch of X; sweeping the stretches from high to low answers the queries in decreasing k.
@!multitest
let (n, q) = read<(1..=5e4, 1..=5e4)>()
let a: [n]1..=1e9 = read()
let ks: [q]1..=1e9 = read()
let window = a.prefix()
let cuts = compress(a + a.map(x => 2 * x))
let cap = ks.max() + a.max()

// Σ_{j<l} ⌈x / 2^j⌉ = l + 2y − ⌊y / 2^(l−1)⌋ − popcount(y mod 2^(l−1)) with y = x − 1
fn raise(l: 1..=31, x: Int) = let y = x - 1 in l + 2 * y - (y >> (l - 1)) - (y % (1 << (l - 1))).popcount

var ans: [q]Int = a.max()
for l in 1..=31:
    var saved = IntervalMap<cuts.D, ?Int>(none)
    for s in (0..=n - l).sorted(by: s => window[s..s + l]):
        saved[cuts.at(a[s])..(if s > 0: cuts.at(2 * a[s - 1]) else: cuts.end)] = window[s..s + l]
    var waiting = (0..q).sorted(by: ks)
    for (r, some best) in saved.runs.reversed():
        let xs = (cuts[r.first] << (l - 1)) <..= min(cap, (cuts.get(r.end) ?? cap) << (l - 1))
        for x in waiting.pop_while(x => ks[x] + best >= raise(l, xs.first)):
            ans[x] max= xs.last(X => raise(l, X) <= ks[x] + best)
print(ans)
