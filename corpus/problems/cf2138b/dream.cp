// Codeforces 2138 B — Antiamuny Wants to Learn Swap
//
// A subarray `a[l..=r]` is sortable in the minimum number of operations iff it
// contains no 3-term decreasing subsequence `a[x] > a[y] > a[z]` (`l <= x < y < z <= r`).
// For each middle index `i`, the tightest such triple uses its nearest greater
// element to the left `l = prev_gt[i]` and nearest smaller element to the right
// `r = next_lt[i]`, so any query starting at or before `l` fails once it reaches `r`.
// Taking suffix minima of `bad[l]` over `I` gives `min_bad[l]`, the earliest
// forbidden right endpoint for any subarray starting at `l`.


@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]1..=500_000 = read()

let prev_gt = a.prev_smaller(by: desc, strict: false)
let next_lt = a.next_smaller(strict: false)
var min_bad: [I]?I = None
for i in I:
    if let l = prev_gt[i] and let r = next_lt[i]:
        min_bad[l] min= r
for i in I.rev:
    if let p = i.prev and let r = min_bad[i]:
        min_bad[p] min= r

for _ in 0..q:
    let (l, r): (I, I) = read()
    print(if let b = min_bad[l] and r >= b: "NO" else: "YES")
