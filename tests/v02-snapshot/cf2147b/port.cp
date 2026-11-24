// Source: Codeforces 2147B (Multiple Construction), https://codeforces.com/contest/2147/problem/B
// Problem: given n <= 2e5, print an array of length 2n holding each of 1..=n twice, the two copies
//   of x a multiple of x apart. Multitest, sum n <= 2e5.
// Solution: print n - 1, ..., 1, n, 1, ..., n - 1, n: each x < n sits x places either side of the
//   first n, so its copies are 2x apart, and the two n's are n apart. O(n).

@!multitest
let n: 1..=200_000 = read()
let desc = (1..n).rev
print(desc + [n] + desc.rev + [n])
