// Codeforces 2137 E — Mexification
//
// Each step simultaneously replaces every `a[i]` by `MEX(a \ {a[i]})`.
// Let `base` be the smallest non-negative integer with `cnt[base] != 1`; every
// value in `0..base` appears once and is a fixed point forever.
// - If `base >= n - 1`: after one step the array is a permutation of `0..n`
//   and stays fixed with sum `n * (n - 1) / 2`.
// - If `cnt[base] >= 2` and `k == 1`: letting `mex` be the full-array MEX,
//   unique values `< mex` stay unchanged while every duplicate or value `> mex`
//   maps to `mex`.
// - Otherwise (`k >= 2`, or `cnt[base] == 0`): all `n - base` elements `>= base`
//   collapse to one shared value `v` that alternates between `base` and
//   `base + 1` each step (`v = base + 1` iff `(cnt[base] == 0) == (k % 2 == 0)`,
//   else `v = base`).

@!multitest
let (n, k): (1..=200_000, 1..=10 ** 9) = read()
let a: [0..n]0..=n = read()
var cnt: [0..=n]Int = 0
for x in a: cnt[x] += 1

var base = 0
while cnt[base] == 1: base += 1
if base >= n - 1:
    print(n * (n - 1) / 2)
elif cnt[base] >= 2 and k == 1:
    var mex = base
    while cnt[mex] > 0: mex += 1
    print(sum((if x < mex and cnt[x] == 1: x else: mex) for x in a))
else:
    let v = if (cnt[base] == 0) == (k % 2 == 0): base + 1 else: base
    print(sum((if x < base: x else: v) for x in a))
