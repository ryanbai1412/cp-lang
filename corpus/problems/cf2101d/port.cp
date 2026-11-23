// Codeforces 2101 D — Mani and Segments
//
// A subarray a[l..=r] of a permutation is "cute" (LIS + LDS = len + 1) iff
// it contains no element strictly between two earlier elements (1-3-2 / 3-1-2)
// and none strictly between two later elements (2-3-1 / 2-1-3).
//
// For each i, two monotone stacks give the nearest earlier smaller and larger
// indices; if both exist, any start l <= min(prev_lo, prev_hi) breaks at i.
// A suffix minimum turns those into bad_r[l], the first right endpoint that
// breaks start l. Running the same scan on the reversed array gives bad_l[r],
// the last left endpoint that breaks end r.
//
// Since bad_l and bad_r are non-decreasing, the cute subarrays are the union
// of rectangles [bad_l[i] + 1 ..= i] x [i .. bad_r[i]]. Adjacent rectangles
// k and k + 1 overlap on [bad_l[k + 1] + 1 ..= k] x [k + 1 .. bad_r[k]], so
// subtracting adjacent overlaps counts every cute subarray once in O(n).

use std.prefix.{suffix_fold}

@!multitest
let n: 1..=200_000 = read()
domain Pos = 0..n
let a: [Pos]1..=n = read()

fn first_bad(xs: [Pos]Int) -> [Pos]Int:
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
    let suf = suffix_fold(min, cur)
    [Pos] i => suf[i.l]

let bad_r = first_bad([Pos] i => a[i])
let bad_l_rev = first_bad([Pos] i => a[Pos(n - 1 - i.ord)])
let bad_l = [Pos] i => n - 1 - bad_l_rev[Pos(n - 1 - i.ord)]

let total = sum((bad_r[i] - i.ord) * (i.ord - bad_l[i]) for i in Pos)
let overlap = sum((bad_r[k] - (k.ord + 1)) * (k.ord - bad_l[k.next!]) for k in Pos if k < Pos.last!)
print(total - overlap)
