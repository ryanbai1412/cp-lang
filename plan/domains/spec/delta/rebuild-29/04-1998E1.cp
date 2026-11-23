// CF 1998E1 Eliminating Balls With Merging (Easy, x = n): count balls that can survive.
// Split at the maximum: inside a piece bounded by bigger balls, a survivor of the piece
// escapes only if the whole piece's sum reaches the bounding maximum next to it.

@!multitest
let (n, x): (1..=2e5, 1..=2e5) = read()
let a = read<[n]1..=1e9>()
let pre = a.scan(+)
let top = SparseTable((i, j) => if a[j] > a[i]: j else: i, [n] i => i)

fn wins(lo: Int, hi: Int, ok: Bool) -> Int:
    if lo >= hi: return 0
    let m = top.fold(lo..hi)
    let here = if ok: 1 else: 0
    here + wins(lo, m, ok and pre[m] - pre[lo] >= a[m]) + wins(m + 1, hi, ok and pre[hi] - pre[m + 1] >= a[m])

print(wins(0, n, true))
