// B. Problem from Chebarkul
// https://codeforces.com/contest/2289/problem/B
// position i (0-based) takes a[i..=i+n-k] across all subsequences; it is fixed iff that window is constant
@!multitest
let (n, k): (1..=2e5, 1..=n) = read()
let a: [0..n]1..=n = read()
var run: [0..n]Int = 1
for j in (0..n - 1).rev:
    if a[j] == a[j + 1]: run[j] = run[j + 1] + 1
print((0..k).count(i => run[i] >= n - k + 1))
