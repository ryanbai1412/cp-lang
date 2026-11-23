// Codeforces 2268 C: largest v such that some subarray of length >= 2 has
// both its maximum and its xor containing every bit of v. Build v greedily
// from the top bit. For a fixed v, a subarray is a pair of cuts; its masked
// xor is v exactly when the masked prefix xors p, q at its cuts satisfy
// q = p ^ v, i.e. both lie in the class min(p, p ^ v) with opposite parity
// (whether the prefix is the larger of the pair). Within a class, the pair's
// maximum is the largest gap maximum between them, so folding the class's
// gaps from the smallest maximum up (a Cartesian tree of the gaps) meets
// every pair exactly once, at the gap that is its maximum.
@!multitest
domain I = read(1..=2e5)

struct Span(lo: I.bounds, hi: I.bounds):  // leftmost and rightmost cut of one parity
    fn join(self, o: Span) -> Span = Span(min(lo, o.lo), max(hi, o.hi))

// The extreme cuts of each parity on one side of a gap.
struct Ends(even: ?Span, odd: ?Span):
    fn join(self, o: Ends) -> Ends = Ends(even.join(o.even), odd.join(o.odd))
    // some cut here and an opposite-parity cut in `right` are at least 2 apart
    fn reaches(self, right: Ends) -> Bool =
        (right.odd.hi - even.lo ?? 0) > 1 or (right.even.hi - odd.lo ?? 0) > 1

let a: [I]Int = read()
let peak = SparseTable<I, Int>(max, a)

fn feasible(v: Int) -> Bool:
    let pre = (a & v).prefix_fold(xor)
    let class = [I.bounds] c => min(pre[c], pre[c] ^ v)
    let at = c => if pre[c] != class[c]: Ends(None, Span(c, c)) else: Ends(Span(c, c), None)
    I.cuts.sorted(by: c => (class[c], c)).chunk_by(c => class[c]).any(cuts =>
        cuts.cartesian(
            gap: (l, r) => peak.fold(l..r),
            leaf: at,
            join: Ends.join,
        ).any((m, left, right) => m & v == v and left.reaches(right))
    )

var ans = 0
for b in (0..18).rev if feasible(ans | 1 << b): ans |= 1 << b
print(ans)
