// Codeforces 2268 C: largest v such that some subarray of length >= 2 has
// both its maximum and its xor containing every bit of v. Build v greedily
// from the top bit. For a fixed v, boundaries whose masked prefix xors agree
// up to `^ v` form a class with a parity; a subarray works when its two
// boundaries share a class with opposite parity and its maximum contains v.
// Each class is folded along the Cartesian tree of the maxima between
// neighbouring boundaries, keeping the extreme boundary of each parity.
const INF = 1e9

struct Ends(lo0: Int, lo1: Int, hi0: Int, hi1: Int)  // leftmost / rightmost boundary per parity

@!multitest
domain I = read(1..=2e5)
let n = I.count
let a: [I]Int = read()
let peak = SparseTable<I, Int>(max, a)

fn feasible(v: Int) -> Bool:
    var pre = 0
    var bounds: List<(key: Int, pos: Int, par: Int)> = []
    for i in 0..=n:
        let key = min(pre, pre ^ v)
        bounds.push((key, i, Int(pre != key)))
        if i < n: pre ^= a[I(i)] & v
    bounds.sort()

    var stk: List<(top: Int, ends: Ends)> = []
    for (t, b) in bounds.items:
        // maximum up to the next boundary of this class; the last one folds everything
        let top = if t < n and bounds[t + 1].key == b.key:
            peak.fold(I(b.pos)..=I(bounds[t + 1].pos - 1))
        else:
            INF
        var cur = if b.par == 0: Ends(b.pos, INF, b.pos, -INF) else: Ends(INF, b.pos, -INF, b.pos)
        while let s = stk.last, top >= s.top:
            let (m, left) = stk.pop()!
            if m & v == v and (cur.hi1 - left.lo0 > 1 or cur.hi0 - left.lo1 > 1):
                return true
            cur.lo0 min= left.lo0
            cur.lo1 min= left.lo1
            cur.hi0 max= left.hi0
            cur.hi1 max= left.hi1
        if top < INF: stk.push((top, cur))
    false

var ans = 0
for b in (0..18).rev if feasible(ans | 1 << b): ans |= 1 << b
print(ans)
