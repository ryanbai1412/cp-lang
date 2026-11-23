@!multitest
let (n, k): (1..=2e5, 1..=n) = read()
let a: [0..n]1..=n = read()
var run: [0..=n]Int = 0
for i in (0..n).rev:
    run[i] = if i + 1 < n and a[i] == a[i + 1]: run[i + 1] + 1 else: 1
print((0..k).count(i => run[i] >= n - k + 1))
