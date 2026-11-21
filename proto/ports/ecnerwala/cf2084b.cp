@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var mn = a[0]
for i in 1..n:
    mn min= a[i]
var skipped = false
var g = 0
for i in 0..n:
    if a[i] == mn and not skipped:
        skipped = true
    elif a[i] % mn == 0:
        g = gcd(g, a[i])
print(if g == mn: "Yes" else: "No")
