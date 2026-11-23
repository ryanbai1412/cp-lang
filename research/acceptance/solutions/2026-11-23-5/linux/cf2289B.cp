@!multitest
let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=n = read()
let w = n - k + 1
var start = [n].filled(0)
for j in 1..n:
    start[j] = if a[j] == a[j - 1]: start[j - 1] else: j
print((0..k).count(i => start[i + w - 1] <= i))
