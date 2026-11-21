let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
var p: [0..=n]Int = 0
for i in 0..n:
    p[i + 1] = p[i] + a[i]
var best = -1_000_000_000_000
for i in 0..=n - k:
    best = max(best, p[i + k] - p[i])
print(best)
