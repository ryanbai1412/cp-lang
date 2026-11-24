// Source: Codeforces 2137D (Replace with Occurrences),
//   https://codeforces.com/contest/2137/problem/D
// limits: 2000 ms, 256 MB
// Problem: given b of length n <= 2e5 (1 <= b_i <= n), build a with 1 <= a_i <= n where the
//   value a_i occurs exactly b_i times in a, or print -1. Multitest, sum n <= 2e5.
// Solution: positions with b_i = x split into groups of x sharing one label, so a exists iff
//   the count of each x is a multiple of x. One pass: a position with b_i = x joins the open
//   group of x, opening one with a fresh label when there is none, and the group closes once
//   it holds x positions; a exists iff no group is left open. O(n).
// Bounds: n: 1..=200_000; b: 1..=n

@!multitest
let n: 1..=200_000 = read()
let b: [0..n]1..=n = read()
var label: [1..=n]Int = 0
var left: [1..=n]Int = 0
var fresh = 0
var a: [0..n]Int = 0
for (i, x) in b.items:
    if left[x] == 0:
        fresh += 1
        (label[x], left[x]) = (fresh, x)
    a[i] = label[x]
    left[x] -= 1
if (1..=n).any(x => left[x] > 0): print(-1)
else: print(a)
