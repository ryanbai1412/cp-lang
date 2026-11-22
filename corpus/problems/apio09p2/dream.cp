// apio09p2 Convention Center: the most pairwise-disjoint intervals, and the lexicographically
// smallest set of that size.
// Greedy: after interval i the best next pick is the one with the smallest right end that starts
// after it. Those links form a forest, so "how many fit inside a gap" is a depth difference. Free
// gaps are ranges of compressed cuts; taking an interval splits its gap in two, and it is taken
// iff the two halves still fit as many as the whole gap minus one.
let n = read<1..=200_000>()
domain I = 1..=n
let iv = read<[I](l: Int, r: Int)>()
let (domain X, at) = compress(chain(iv.map(x => x.l), iv.map(x => x.r)))
let span: [I]Range<X> = [I] i => at.at(iv[i].l)..=at.at(iv[i].r)

let byl = I.range.sorted(by: i => span[i].l)
let soonest = byl.rev.scan(min(by: i => span[i].r))
fn first_from(c: Cut<X>) -> ?I:
    soonest[byl.partition_point(i => span[i].l < c).r?]
let t = Rooted.from_parents(I.map(i => first_from(span[i].r)).collect())
fn fit(gap: Range<X>) -> Int:
    let u = first_from(gap.l) ?? return 0
    if span[u].r > gap.r:
        return 0
    t.depth(u) - t.depth(t.jump_while(u, v => span[v].r <= gap.r)!) + 1

var gaps = SortedMap<Cut<X>, (gap: Range<X>, best: Int)>()
gaps.set(X.cuts.first!, (gap: X.range, best: fit(X.range)))
var picked = List<I>()
for i in I:
    let (_, g) = gaps.last_le(span[i].l)!
    if span[i].r > g.gap.r:
        continue
    let (left, right) = (g.gap.l..span[i].l, span[i].r..g.gap.r)
    let (a, b) = (fit(left), fit(right))
    if a + 1 + b == g.best:
        picked.push(i)
        gaps.set(left.l, (gap: left, best: a))
        gaps.set(right.l, (gap: right, best: b))
print(picked.len)
print(picked)
