// An optimal subsegment [l, r] always achieves its min and max at its endpoints l and r;
// shrinking past them only reduces r - l. So we maximize over i <= j either
// (-a[i] + i) + (a[j] - j) or (a[i] + i) + (-a[j] - j).
// A segment tree over non-empty intervals tracks the four endpoint terms and the best pair.
@!multitest
let n: 1..=200_000 = read()
let q: 1..=200_000 = read()
domain Pos = 1..=n
let a: [Pos]1..=1_000_000_000 = read()

const NEG = -1e15

struct Span(best: Int, lo_l: Int, hi_l: Int, lo_r: Int, hi_r: Int)

fn leaf(p: Pos, x: Int) -> Span = Span(0, -x + p.ord, x + p.ord, -x - p.ord, x - p.ord)

fn combine(a: Span, b: Span) -> Span = Span(
    max(max(a.best, b.best), max(a.lo_l + b.hi_r, a.hi_l + b.lo_r)),
    max(a.lo_l, b.lo_l),
    max(a.hi_l, b.hi_l),
    max(a.lo_r, b.lo_r),
    max(a.hi_r, b.hi_r),
)

var tree = SegTree<Pos, Span>(
    Monoid(Span(0, NEG, NEG, NEG, NEG), combine),
    [Pos] i => leaf(i, a[i]),
)
print(tree.all.best)
repeat q:
    let (p, x): (Pos, 1..=1_000_000_000) = read()
    tree.set(p, leaf(p, x))
    print(tree.all.best)
