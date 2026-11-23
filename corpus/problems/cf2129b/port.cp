// Source: Codeforces 2129B (Stay or Mirror), https://codeforces.com/contest/2129/problem/B
// Problem: a permutation p of n <= 5000; set each a[i] to p[i] or 2n - p[i]. Print the fewest
//   inversions a can have. Multitest, sum n <= 5000.
// Solution: for p[j] > p[i] both choices for j lie strictly between p[i] and 2n - p[i], so the
//   pair is an inversion iff i stays and j is left of i, or i mirrors and j is right of it. So
//   each i picks alone: the answer is the sum over i of min(larger before i, larger after i),
//   counted directly in O(n^2).

@!multitest
domain I = read(1..=5_000)
let p: [I]1..=5_000 = read()
print(sum(min(I.before(i).count(j => p[j] > p[i]), I.after(i).count(j => p[j] > p[i])) for i in I))
