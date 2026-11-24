// Source: Codeforces 2057D (Gifts Order), https://codeforces.com/contest/2057/problem/D
// limits: 2000 ms, 500 MB
// Problem: an array a of n <= 2e5 values <= 1e9. Print the maximum over l <= r of
//   max(a[l..=r]) - min(a[l..=r]) - (r - l), initially and after each of q <= 2e5 point
//   assignments. Multitest, sums of n and q <= 2e5.
// Solution: an optimal segment has its max and min at its two ends (else shrink it), so maximize
//   (a_j - j) - (a_i - i) or (a_i + i) - (a_j + j) over i <= j. A SegTree keeps the best of each
//   endpoint term per side and the best pair, combined across the midpoint. O((n + q) log n).
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
