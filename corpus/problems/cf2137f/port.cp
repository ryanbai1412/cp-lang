// Codeforces 2137 F — Prefix Maximum Invariance
//
// Position `i` can match `b[i]` inside subarray `[L..=R]` (`0 <= L <= i <= R < n`)
// iff either `a[i] == b[i]` (already matches for all `L <= i`, so `lo = i`), or
// some element `a[j]` with `L <= j < i` satisfies `a[j] >= max(a[i], b[i])` (so
// `i` is not a strict prefix maximum and can be rewritten to `b[i]` without
// altering prefix maxima).
//
// Maintaining a strictly decreasing monotone stack `stk` of `(val: a[j], idx: j)`
// over `j < i`, the rightmost qualifying `j` is found by `bsearch_last` for
// `stk[m].val >= max(a[i], b[i])`. Every `L in 0..=lo` and `R in i..n` forms a
// valid subarray, contributing `(lo + 1) * (n - i)` to `ans`.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=2 * n = read()
let b: [0..n]1..=2 * n = read()

var stk: List<(val: Int, idx: Int)> = []
var ans = 0
for i in 0..n:
    let lo: ?Int = if a[i] == b[i]:
        i
    else:
        let need = max(a[i], b[i])
        if let j = (0..stk.len).last_true(m => stk[m].val >= need): stk[j].idx else: none
    if let l = lo: ans += (l + 1) * (n - i)
    while let top = stk.last and top.val <= a[i]: stk.pop()
    stk.push((a[i], i))

print(ans)
