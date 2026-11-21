// CCO '18 P6 — Flop Sorting
// original: corpus/submissions/4575245.cpp
//
// Sort any permutation to 0..n using adjacent-free reversals via divide-and-
// conquer in-place merge: pick the median of a[l..r], binary-search its cut
// in both sorted halves, rotate the middle two blocks with three reversals,
// and recurse. Transform A -> sorted -> B by reversing B's swap log.

let n: 1..=4096 = read()
domain I = 0..n
let a0: [I]Int = read()
let b0: [I]Int = read()

fn sort_swaps(init: [I]Int) -> List<(Int, Int)>:
    var a: List<Int> = [init[i] for i in I]
    var ops: List<(Int, Int)> = []

    fn flip(l0: Int, r0: Int):
        var l = l0
        var r = r0 - 1
        while l < r:
            (a[l], a[r]) = (a[r], a[l])
            ops.push((l, r))
            l += 1
            r -= 1

    fn merge(l: Int, mid: Int, r: Int):
        if l == mid or mid == r: return
        let med = a[l..r].sorted()[(r - l) / 2]
        let m1 = l + a[l..mid].first_ge(med)
        let m2 = mid + a[mid..r].first_ge(med)
        flip(m1, mid)
        flip(mid, m2)
        flip(m1, m2)
        let (len1, len2, len3, len4) = (m1 - l, mid - m1, m2 - mid, r - m2)
        merge(l, l + len1, l + len1 + len3)
        merge(r - len4 - len2, r - len4, r)

    fn dnc(l: Int, r: Int):
        if r - l <= 1: return
        let mid = (l + r) / 2
        dnc(l, mid)
        dnc(mid, r)
        merge(l, mid, r)

    dnc(0, n)
    ops

let fwd = sort_swaps(a0)
let bwd = sort_swaps(b0)
bwd.rev

print(fwd.len + bwd.len)
for (x, y) in fwd: print(x + 1, y + 1)
for (x, y) in bwd: print(x + 1, y + 1)
