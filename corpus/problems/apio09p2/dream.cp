// APIO 2009 Convention Center: the most pairwise-disjoint requests, and the lexicographically
// smallest list of that size. After request i the greedy next pick is the one with the least end
// that starts after it; those links form a forest, so how many fit inside a free gap is a climb.
// Try requests in order: take one iff the gaps left and right of it fit the gap's count minus one.
let n = read<1..=2e5>()
domain I = 1..=n
domain X = 1..=1e9
let iv = read<[I]Range<X>>()

let by_start = I.sort(by: i => iv[i].l)
let soonest = by_start.map(i => Some((iv[i].r, i))).rev.scan(min)
fn first_from(c: Cut<X>) -> ?I = soonest[by_start.bsearch(i => iv[i].l >= c)]?.1
let chain = Lift<I, ()>.of(i => first_from(iv[i].r))
fn fit(gap: Range<X>) -> Int:
    let u = first_from(gap.l) ?? return 0
    if iv[u].r > gap.r:
        return 0
    1 + chain.climb(u, ok: v => iv[v].r <= gap.r).steps

var gaps = IntervalMap<X, ?Int>(init: None)
gaps.assign(X.range, fit(X.range))
var picked = List<I>()
for i in I:
    let (gap, best) = gaps.run_at(iv[i].first!)
    let best = best ?? continue
    if gap.clip(iv[i]) != iv[i]:
        continue
    let (left, right) = (gap.l..iv[i].l, iv[i].r..gap.r)
    let (a, b) = (fit(left), fit(right))
    if a + 1 + b == best:
        picked.push(i)
        gaps.assign(left, a)
        gaps.assign(iv[i], None)
        gaps.assign(right, b)
print(picked.len)
print(picked)
