// Source: Codeforces 2268C (KiaKio and Energy Intervals),
//   https://codeforces.com/contest/2268/problem/C
// Problem: an array of n >= 2 values a_i < 2^18. Over segments l < r with maximum m, print the
//   largest xor of (a_i & m). Multitest, sum of n <= 2e5.
// Solution: build the answer v from bit 17 down, keeping a bit when some segment has a maximum
//   containing v and xor of a_i & v equal to v. Such a segment is two cuts at least 2 apart
//   whose masked prefix xors p, p ^ v share the class min(p, p ^ v) with opposite parity. Per
//   class, a monotonic stack over the gaps between consecutive cuts (gap maxima by SparseTable)
//   merges groups at each gap's maximum, the maximum of every pair it joins. O(18 n log n).
use std.prefix.{prefix_fold}

@!multitest
domain I = read(1..=2e5)

struct Span(lo: I.bounds, hi: I.bounds)  // leftmost and rightmost cut of one parity

fn hull(x: ?Span, y: ?Span) -> ?Span:
    if let s = x and let t = y: Span(min(s.lo, t.lo), max(s.hi, t.hi)) else: x ?? y

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
