// A running difference of two different sequences is not a window sum.
let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
var cur = 0
var best = 0
for i in k..n:
    cur += a[i] - b[i - k]
    best max= cur
print(best)
