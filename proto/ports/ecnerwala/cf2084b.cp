use std.seq.{argmin}

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let p = argmin(a)!
let mn = a[p]
var g = 0
for i in 0..n if i != p and a[i] % mn == 0: g = gcd(g, a[i])
print(if g == mn: "Yes" else: "No")
