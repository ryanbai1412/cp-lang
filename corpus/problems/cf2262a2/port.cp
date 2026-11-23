// Codeforces 2262 A2 — Bucket-Avoidance Subset Counting
//
// Each `a[i]` satisfies `0 <= a[i] <= ceil(n / i)`: every `j < a[i]` has
// `j * i < n` (visiting `sum_{i=1..n} ceil(n / i) = O(n log n)` must-hit
// buckets `[j * i, min(n, (j + 1) * i))`), while `a[i] * i` may exceed `n`
// (when `i` does not divide `n`), so both endpoints of the forbidden bucket
// `[a[i] * i, (a[i] + 1) * i)` are clamped to `n` in `diff`.
//
// `diff` is a range-add difference array over forbidden buckets
// `[a[i] * i, min(n, (a[i] + 1) * i))`, so `cover.prefix(i + 1)` counts how
// many forbidden buckets cover `i`. `need[lo]` starts at `n + 1` (meaning no
// must-hit interval constrains `lo`) and after `suffix_scan_in_place(..., min)`
// holds the tightest right endpoint of any must-hit interval starting at `>= lo`.
//
// Scanning `i` from `n - 1` down to `0`, `ways[i]` counts valid subsets whose
// smallest element is `i`: if `i` is uncovered (`cover.prefix(i + 1) == 0`),
// the next chosen element must lie in `i + 1..need[i]`, whose running sum is
// maintained in `tot` by shrinking `hi` down to `need[i]`.

use std.prefix.{suffix_fold}

const MOD = 1e9 + 7
type Z = Mod<MOD>

@!multitest
let n: 1..=2e5 = read()
let a: [1..=n]0..=n = read()
var diff: [0..=n]Int = 0
var need: [0..=n]Int = n + 1
for i in 1..=n:
    for j in 0..a[i]: need[j * i] min= min(n, j * i + i)
    diff[min(n, a[i] * i)] += 1
    diff[min(n, a[i] * i + i)] -= 1
let cover = prefix(diff)
let need_suf = suffix_fold(min, need)

var ways: [0..=n]Z = 0
ways[n] = 1
var tot: Z = 1
var hi = n + 1
for i in (0..n).rev:
    if cover.prefix(i + 1) == 0:
        ways[i] = tot
        tot += ways[i]
    while hi > need_suf[i]:
        hi -= 1
        tot -= ways[hi]
print(tot)
