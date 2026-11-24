// Source: Codeforces 2219A (Grid L), https://codeforces.com/contest/2219/problem/A
// limits: 2000 ms, 256 MB
// Problem: p unit segments and q L-shaped pieces (two unit segments at a right angle),
//   1 <= p, q <= 1e8; print positive n, m such that all the pieces exactly form the segments
//   of an n x m grid, or -1. Multitest, t <= 100.
// Solution: an n x m grid has 2nm + n + m unit segments, so p + 2q = 2nm + n + m, i.e.
//   (2n + 1)(2m + 1) = 2(p + 2q) + 1. Try the odd divisors 2n + 1 >= 3 up to the square root
//   (n <= m); each L uses one horizontal and one vertical segment, so the pair fits iff
//   q <= nm + n, the smaller orientation count. O(sqrt(p + q)).
// Bounds: p, q: 0..=10 ** 9

@!multitest
let (p, q): (0..=10 ** 9, 0..=10 ** 9) = read()
let prod = (p + 2 * q) * 2 + 1
for i in 3..=isqrt(prod) by 2 if prod % i == 0:
    let (n, m) = (i / 2, prod / i / 2)
    if q <= n * m + n:
        print(n, m)
        return
print(-1)
