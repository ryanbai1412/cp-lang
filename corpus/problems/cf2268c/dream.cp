// Codeforces 2268 C: largest v such that some subarray of length >= 2 has
// both its maximum and its xor containing every bit of v. Build v greedily
// from the top bit. For a fixed v, a subarray is a pair of cuts; its masked
// xor is v exactly when the masked prefix xors p, q at its cuts satisfy
// q = p ^ v, i.e. both lie in the class min(p, p ^ v) with opposite parity
// (whether the prefix is the larger of the pair). Within a class, the pair's
// maximum is the largest gap maximum between them, so folding the class's
// gaps from the smallest maximum up (a Cartesian tree of the gaps) meets
// every pair exactly once, at the gap that is its maximum.
// copies: 1
use std.prefix.{prefix_fold}
@!multitest
domain I = read(1..=2e5)

struct Span(lo: I.bounds, hi: I.bounds)  // leftmost and rightmost cut of one parity

fn hull(x: ?Span, y: ?Span) -> ?Span:
    if let s = x and let t = y: Span(min(s.lo, t.lo), max(s.hi, t.hi)) else: x ?? y

fn wide(left: ?Span, right: ?Span) -> Bool:
    if let l = left and let r = right: r.hi.ord - l.lo.ord > 1 else: false

struct Ends(even: ?Span, odd: ?Span):
    fn join(self, o: Ends) -> Ends = Ends(hull(even, o.even), hull(odd, o.odd))
    // some cut here and an opposite-parity cut in `right` are at least 2 apart
    fn reaches(self, right: Ends) -> Bool = wide(even, right.odd) or wide(odd, right.even)

let a: [I]Int = read()
let peak = SparseTable<I, Int>(max, a)

fn feasible(v: Int) -> Bool:
    let pre = prefix_fold(^, [I] i => a[i] & v)
    let class = [I.bounds] c => min(pre[c], pre[c] ^ v)
    fn at(c: I.bounds) -> Ends:
        if pre[c] != class[c]: Ends(None, Span(c, c)) else: Ends(Span(c, c), None)

    // One class's cuts in order: split at the highest gap first, as a max Cartesian tree over the gaps.
    fn splits(cuts: List<I.bounds>) -> Bool:
        let gaps = [peak.fold(cuts[j]..cuts[j + 1]) for j in 0..cuts.len - 1]
        let tree = gaps.cartesian_tree(min_root: false)
        fn walk(node: ?Int, leaf: Int) -> (ends: Ends, found: Bool):
            guard let g = node else: return (at(cuts[leaf]), false)
            let (le, lf) = walk(tree.left[g], g)
            let (re, rf) = walk(tree.right[g], g + 1)
            (le.join(re), lf or rf or (gaps[g] & v == v and le.reaches(re)))
        walk(tree.root, 0).found

    I.cuts.sorted(by: c => (class[c], c)).runs(by: c => class[c]).any(run => splits(run.collect()))

var ans = 0
for b in (0..18).rev if feasible(ans | 1 << b): ans |= 1 << b
print(ans)
