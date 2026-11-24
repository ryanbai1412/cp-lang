// Source: Codeforces 2180C (XOR-factorization), https://codeforces.com/contest/2180/problem/C
// limits: 2000 ms, 256 MB
// Problem: given n <= 1e9 and k <= 1e5, print k integers in 0..=n whose XOR is n and whose sum is
//   as large as possible. Multitest, sum of k <= 1e5.
// Solution: odd k: k copies of n. Even k: go from the top bit down, keeping a prefix of the
//   elements already below n (free on lower bits) while the rest still match n. A set bit of n
//   goes to k - 1 elements: the one left out is newly freed, or the last one when all are free.
//   A clear bit goes to the largest even number of free elements. O(31 k).
// Bounds: n: 1..=10 ** 9; k: 1..=100_000

const MAX_BIT = 30

@!multitest
let (n, k): (1..=10 ** 9, 1..=100_000) = read()
if k % 2 == 1:
    print(List.filled(k, n))
    return

var a: [0..k]Int = 0
var num_smaller = 0
for b in (0..=MAX_BIT).rev:
    if n.bit(b):
        if num_smaller < k: num_smaller += 1
        a[0..k] += 1 << b
        a[num_smaller - 1] -= 1 << b
    else:
        a[0..num_smaller / 2 * 2] += 1 << b
print(a)
