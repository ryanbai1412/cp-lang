// Source: Codeforces 2262A1 (Floor of MEX, easy), https://codeforces.com/contest/2262/problem/A1
// limits: 2000 ms, 256 MB
// Problem: f(S, x) = mex of {floor(y / x) : y in S}. Given a_k = f(A, k) for k = 1..=n (n <= 1e5,
//   0 <= a_k <= n) of a hidden A within 0..n, print any B within 0..n with f(B, k) = a_k for
//   all k: its size, then its elements. Multitest, sum n <= 1e5.
// Solution: f(B, k) = a_k needs no y in the bucket [a_k k, (a_k + 1) k) and every lower bucket
//   hit. B = every y in 0..n outside all those forbidden buckets contains the valid A, so it
//   hits every bucket A hits. Mark the buckets, clamped to 0..=n, in a difference array: O(n).
// Bounds: n: 1..=200_000; a: 0..=10 ** 9

@!multitest
let n: 1..=200_000 = read()
let a: [1..=n]0..=10 ** 9 = read()
fn cap(x: Int) -> Int: min(n, x)

var diff: [0..=n]Int = 0
for i in 1..=n:
    diff[cap(a[i] * i)] += 1
    diff[cap((a[i] + 1) * i)] -= 1
let cover = scan(diff, +)
let free = [i for i in 0..n if cover.prefix(i + 1) == 0]
print(free.len)
print(free)
