@!multitest
let n: Int = read()
let m: Int = read()
let k: Int = read()
var a: [0..n]Int = 0
var cur = 0
for i in 0..k:
    var v = (n - 1 - i) % k
    while v < n:
        a[v] = cur / (m + 1)
        cur += 1
        v += k
print(a)
