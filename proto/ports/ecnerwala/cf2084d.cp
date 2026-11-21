@!multitest
let (n, m, k): (Int, Int, Int) = read()
var a: [0..n]Int = 0
var cur = 0
for i in 0..k:
    for v in (n - 1 - i) % k..n by k:
        a[v] = cur / (m + 1)
        cur += 1
print(a)
