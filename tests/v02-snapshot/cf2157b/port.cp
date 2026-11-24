// Source: Codeforces 2157B (Expansion Plan 2), https://codeforces.com/contest/2157/problem/B
// Problem: only (0, 0) of an infinite grid is black; a string of n <= 2e5 steps follows: '4'
//   blackens the side neighbours of black cells, '8' the side and corner ones. Print YES if (x, y)
//   (|x|, |y| <= 1e9) ends black. Multitest, sum n <= 2e5.
// Solution: the black set is a Minkowski sum, so order does not matter: c8 '8' steps make a
//   Chebyshev square and each of the c4 = n - c8 '4' steps adds one Manhattan unit. So (x, y) is
//   black iff max(0, |x| - c8) + max(0, |y| - c8) <= c4. Count the '8's, O(n).

@!multitest
let (n, x0, y0): (1..=200_000, -10 ** 9..=10 ** 9, -10 ** 9..=10 ** 9) = read()
let s: [0..n]Char = read()
let c8 = s.count(c => c == '8')
let c4 = n - c8
print(yes(max(0, abs(x0) - c8) + max(0, abs(y0) - c8) <= c4))
