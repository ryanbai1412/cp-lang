// Codeforces 2268 A: the middle block a[k-1 .. n-k] is always taken; outside
// it the positions pair up from both ends and each pair gives its larger value.
@!multitest
let (n, k): (1..=2e5, Int) = read()
let a: [0..n]Int = read()
let mid = if 2 * k - 2 <= n: sum(a[k - 1..n - k + 1]) else: 0
let edge = min(min(k - 1, n - k + 1), n / 2)
print(mid + sum(max(a[i], a[n - 1 - i]) for i in 0..edge))
