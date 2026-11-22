// Codeforces 2262 A2 — Bucket-Avoidance Subset Counting
//
// Count subsets of `0..n` that avoid every forbidden bucket
// `[a[i] * i, (a[i] + 1) * i)` (marked via difference array `diff` / prefix
// sum `cover`) yet hit every lower bucket `[j * i, (j + 1) * i)` for
// `j < a[i]`.
//
// After suffix-minimizing `need`, `need[lo]` is the tightest right endpoint
// of any must-hit interval starting at or after `lo`. Scanning `i` from `n - 1`
// down to `0`, `ways[i]` counts valid subsets whose smallest element is `i`:
// if `i` is uncovered (`cover.prefix(i + 1) == 0`), the next chosen element
// must lie in `i + 1..need[i]`, whose running sum is maintained in `tot` by
// shrinking `hi` down to `need[i]`.

use std.seq.{suffix_scan_in_place}

const MOD = 1e9 + 7
type Z = Mod<MOD>

@!multitest
let n: 1..=2e5 = read()
let a: [1..=n]Int = read()
var diff: [0..=n]Int = 0
var need: [0..=n]Int = n + 1
for i in 1..=n:
    for j in 0..a[i]: need[j * i] min= min(n, j * i + i)
    diff[min(n, a[i] * i)] += 1
    diff[min(n, a[i] * i + i)] -= 1
let cover = prefix(diff)
suffix_scan_in_place(mut need, min)

var ways: [0..=n]Z = 0
ways[n] = 1
var tot: Z = 1
var hi = n + 1
for i in (0..n).rev:
    if cover.prefix(i + 1) == 0:
        ways[i] = tot
        tot += ways[i]
    while hi > need[i]:
        hi -= 1
        tot -= ways[hi]
print(tot)
