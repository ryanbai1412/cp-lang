// Source: Codeforces 2137E, Mexification, https://codeforces.com/contest/2137/problem/E
// limits: 2000 ms, 256 MB
// Problem: multitest, sum n <= 2e5, a_i in 0..=n, k <= 1e9. Apply k times: replace every a_i
//   at once by the MEX of the other elements. Print the sum of the final array.
// Solution: let base be the least value whose count is not 1; values below it stay fixed. If
//   base >= n - 1 the array becomes the permutation 0..n. If base is duplicated and k == 1,
//   unique values below the full MEX stay and the rest become the MEX. Otherwise the n - base
//   others all equal one value alternating base, base + 1 with k's parity. O(n).
// Bounds: n: 1..=200_000; k: 1..=10 ** 9; a: 0..=n

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
