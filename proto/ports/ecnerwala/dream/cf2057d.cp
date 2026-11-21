// An optimal subsegment [l, r] always achieves its min and max at its endpoints l and r;
// shrinking past them only reduces r - l. So we maximize over i <= j either
// (-a[i] + i) + (a[j] - j) or (a[i] + i) + (-a[j] - j).
// A segment tree over non-empty intervals tracks the four endpoint terms and the best pair.
@!multitest
let (n, q): (1..=200_000, 1..=200_000) = read()
domain Pos = 1..=n
let a: [Pos]1..=1_000_000_000 = read()

struct Span(best: Int, lo_l: Int, hi_l: Int, lo_r: Int, hi_r: Int):
    fn at(p: Pos, x: Int) -> Span = Span(0, -x + p.ord, x + p.ord, -x - p.ord, x - p.ord)

    fn +(a: Span, b: Span) -> Span = Span(
        max(a.best, b.best, a.lo_l + b.hi_r, a.hi_l + b.lo_r),
        max(a.lo_l, b.lo_l),
        max(a.hi_l, b.hi_l),
        max(a.lo_r, b.lo_r),
        max(a.hi_r, b.hi_r),
    )

var tree = SegTree<Pos, Span>([Pos] i => Span.at(i, a[i]))
print(tree.all.best)
repeat q:
    let (p, x): (Pos, 1..=1_000_000_000) = read()
    tree[p] = Span.at(p, x)
    print(tree.all.best)
