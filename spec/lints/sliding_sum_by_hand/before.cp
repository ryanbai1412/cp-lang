let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var cur = sum(a[i] for i in 0..min(k, n))
var best = cur
for i in k..n:
    cur += a[i] - a[i - k]
    best max= cur
print(best)
