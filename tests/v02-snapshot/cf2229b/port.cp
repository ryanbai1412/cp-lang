// Source: Codeforces 2229B (Absolute Cinema), https://codeforces.com/contest/2229/problem/B
// Problem: arrays a and b of n <= 1e5 values in 1..=1e9; any a[i] and b[i] may be swapped. Print
//   the largest max(a) + sum(b). Multitest, sum n <= 1e5.
// Solution: the pair holding max(a) adds both its values and every other pair adds its b, best its
//   larger value. So the answer is the sum of max(a[i], b[i]) plus the largest min(a[i], b[i])
//   (that pair puts its larger value in a, which is then max(a)). O(n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let b: [0..n]1..=10 ** 9 = read()
let pairs = [(min(x, y), max(x, y)) for (x, y) in zip(a, b)]
print(sum(hi for (_, hi) in pairs) + pairs.map((lo, _) => lo).max()!)
