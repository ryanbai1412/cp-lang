// Codeforces 2268 C: largest v such that some subarray of length >= 2 has
// both its maximum and its xor containing every bit of v. Build v greedily
// from the top bit. For a fixed v, a subarray is a pair of cuts; its masked
// xor is v exactly when the masked prefix xors p, q at its cuts satisfy
// q = p ^ v, i.e. both lie in the class min(p, p ^ v) with opposite parity
// (whether the prefix is the larger of the pair). Within a class, the pair's
// maximum is the largest gap maximum between them, so folding the class's
// gaps from the smallest maximum up (a Cartesian tree of the gaps) meets
// every pair exactly once, at the gap that is its maximum.
use std.prefix.{prefix_fold}

@!multitest
domain I = read(1..=2e5)

struct Span(lo: I.bounds, hi: I.bounds)  // leftmost and rightmost cut of one parity

fn hull(x: ?Span, y: ?Span) -> ?Span:
    if let a = x and let b = y: Span(min(a.lo, b.lo), max(a.hi, b.hi)) else: x ?? y

fn wide(left: ?Span, right: ?Span) -> Bool:
    if let l = left and let r = right: r.hi.ord - l.lo.ord > 1 else: false

// The extreme cuts of each parity on one side of a gap.
struct Ends(even: ?Span, odd: ?Span):
    fn join(self, o: Ends) -> Ends = Ends(hull(even, o.even), hull(odd, o.odd))
    // some cut here and an opposite-parity cut in `right` are at least 2 apart
    fn reaches(self, right: Ends) -> Bool = wide(even, right.odd) or wide(odd, right.even)

let a: [I]Int = read()
let peak = SparseTable<I, Int>(max, a)

fn feasible(v: Int) -> Bool:
    let pre = prefix_fold((x, y) => x ^ y, [I] i => a[i] & v)
    var cuts = [(class: min(pre[c], pre[c] ^ v), at: c, odd: (pre[c] ^ v) < pre[c]) for c in I.cuts]
    cuts.sort()
    var stk: List<(top: Int, ends: Ends)> = []
    for (t, b) in cuts.items:
        let here = Span(b.at, b.at)
        var ends = if b.odd: Ends(None, here) else: Ends(here, None)
        let next = cuts.get(t + 1).filter(x => x.class == b.class)
        let top = next.map(x => peak.fold(b.at..x.at))
        while let s = stk.last and (top ?? s.top) >= s.top:
            let (m, left) = stk.pop()!
            if m & v == v and left.reaches(ends):
                return true
            ends = left.join(ends)
        if let m = top: stk.push((m, ends))
    false

var ans = 0
for b in (0..18).rev if feasible(ans | 1 << b): ans |= 1 << b
print(ans)
