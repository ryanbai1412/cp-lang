// Codeforces 2101 D — Mani and Segments
//
// A subarray a[l..=r] of a permutation is "cute" (LIS + LDS = len + 1) iff
// it can be split at some pivot i into a monotone-prefix and monotone-suffix
// relation: no element in [l..=r] lies strictly between two earlier elements
// (no 1-3-2 / 3-1-2) and none lies strictly between two later elements (no
// 2-3-1 / 2-1-3).
//
// For each i, let prev_lo[i] and prev_hi[i] be the nearest earlier positions
// with smaller / larger value (found by two monotone stacks). If both exist,
// starting at or before b = min(prev_lo[i], prev_hi[i]) includes a[i] strictly
// between two earlier elements, so bad_r[l] = min_{b >= l} i (suffix min) is
// the first right endpoint that breaks a start l. Symmetrically, running the
// same scan on the reversed array gives bad_l[r], the last left endpoint that
// breaks an end r.
//
// Because bad_l and bad_r are both non-decreasing, the cute pairs (l, r) are
// the union of the monotone rectangles [bad_l[i] + 1 ..= i] x [i .. bad_r[i]]
// over all i in Pos. Adjacent rectangles k and k + 1 overlap on
// [bad_l[k + 1] + 1 ..= k] x [k + 1 .. bad_r[k]], so inclusion-exclusion over
// adjacent pairs counts every cute subarray exactly once in O(n).

fn first_bad(xs: [Pos]Int) -> [Pos]Int:
    let n = xs.len
    var cur: [Pos]Int = n
    var lo: List<(val: Int, pos: Int)> = []
    var hi: List<(val: Int, pos: Int)> = []
    for i in Pos:
        let v = xs[i]
        while let top = lo.last and v < top.val: lo.pop()
        while let top = hi.last and v > top.val: hi.pop()
        if let x = lo.last and let y = hi.last:
            cur[Pos(min(x.pos, y.pos))] min= i.ord
        lo.push((v, i.ord))
        hi.push((v, i.ord))
    cur.suffix_scan_in_place(min)
    cur

@!multitest
let n: 1..=200_000 = read()
domain Pos = 0..n
let a: [Pos]1..=n = read()

let bad_r = first_bad([Pos] i => a[i])
let bad_l_rev = first_bad([Pos] i => a[Pos(n - 1 - i.ord)])
let bad_l = [Pos] i => n - 1 - bad_l_rev[Pos(n - 1 - i.ord)]

let total = sum((bad_r[i] - i.ord) * (i.ord - bad_l[i]) for i in Pos)
let overlap = sum((bad_r[k] - (k.ord + 1)) * (k.ord - bad_l[k.next!]) for k in Pos if k < Pos.last)
print(total - overlap)
