// Source: Codeforces 2262A2 (Floor of MEX, Hard Version),
//   https://codeforces.com/contest/2262/problem/A2
// limits: 2000 ms, 256 MB
// Problem: f(B, k) = mex of {floor(y / k) : y in B}. Given a[1..n] (0 <= a[k] <= n), count sets
//   B of {0..n-1} with f(B, k) = a[k] for all k, mod 1e9+7 (one is guaranteed to exist).
//   n, sum n <= 1e5; multitest.
// Solution: a[k] says each bucket [j k, (j+1) k) with j < a[k] holds an element of B and the
//   bucket [a[k] k, (a[k]+1) k) holds none. Forbidden buckets are added to a difference array
//   (`cover`); for must-hit buckets, `need_suf[i]` is the least right end of one starting at
//   >= i. ways[i] counts valid sets whose smallest element is i (ways[n] = 1 means none left):
//   the next element lies in i+1 ..< need_suf[i], a window kept as a running sum `tot` while
//   scanning i down. The buckets total sum ceil(n / k) = O(n log n).

use std.prefix.{suffix_fold}

const MOD = 1e9 + 7
type Z = Mod<MOD>

@!multitest
let n: 1..=1e5 = read()
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
    if cover.cut(i + 1) == 0:
        ways[i] = tot
        tot += ways[i]
    while hi > need_suf[i]:
        hi -= 1
        tot -= ways[hi]
print(tot)
