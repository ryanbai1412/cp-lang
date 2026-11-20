// Codeforces 2268 A: the middle block a[k-1 .. n-k] is always taken; outside
// it the positions pair up from both ends and each pair gives its larger value.
@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let a: [0..n]Int = read()
let edge = min(k - 1, n - k + 1)
print(sum(a[i] for i in k - 1..n - k + 1) + sum(max(a[i], a[n - 1 - i]) for i in 0..edge if i < n - 1 - i))
