// Source: Codeforces 2183C (War Strategy), https://codeforces.com/contest/2183/problem/C
// limits: 2000 ms, 256 MB
// Problem: n <= 1e5 bases in a line, one soldier at home base k. Each of m <= 1e9 days, move any
//   soldiers of one base one step, all the same way, then a new soldier appears at k. Print
//   the most bases holding a soldier after day m. Multitest, sum n <= 2e5.
// Solution: with l, r the distances from k to the near and far ends, covering a <= l cells on
//   one side and b <= r on the other takes max(2a + b, a + 2b) - 1 days. With M = m + 1:
//   all n if l + 2r <= M; else, if 3l <= M, the whole near side and (M - l) / 2 far cells;
//   else balance the sides for 2M / 3 cells. Plus the home base. O(1).

@!multitest
let (n, m_in, k): (1..=10 ** 9, 1..=10 ** 9, 1..=10 ** 9) = read()
let m = m_in + 1
let (l, r) = (min(k - 1, n - k), max(k - 1, n - k))
let ans = if l + 2 * r <= m: l + r + 1
elif 3 * l <= m: l + (m - l) / 2 + 1
else: 2 * m / 3 + 1
print(ans)
