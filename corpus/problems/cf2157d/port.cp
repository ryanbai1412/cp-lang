// Source: Codeforces 2157D (Billion Players Game), https://codeforces.com/contest/2157/problem/D
// Problem: an unknown p lies in [l, r], r <= 1e9. Each of n <= 2e5 offers a_i <= 1e9 is ignored
//   or accepted as "p <= a_i" or "p >= a_i", earning |p - a_i| if right and losing it if wrong.
//   Print the most robocoins you can guarantee (minimum over p). Multitest, sum n <= 2e5.
// Solution: claiming p >= x and p <= y for x <= y earns y - x whatever p is. So sort and pair
//   the i-th smallest a with the i-th largest: a pair straddling [l, r] earns its gap; a pair
//   entirely below l claims both ">=" (l - a each), entirely above r both "<=" (a - r each).
//   The term max(0, max(l, a[n-1-i]) - min(r, a[i])) over all i gives exactly these amounts,
//   the self-paired middle included. O(n log n) for the sort.

@!multitest
let (n, l, r): (1..=200_000, 1..=10 ** 9, 1..=10 ** 9) = read()
let raw: [0..n]1..=10 ** 9 = read()
let a = raw.sorted()
print(sum(max(0, max(l, a[n - 1 - i]) - min(r, a[i])) for i in 0..n))
