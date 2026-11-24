// error: `k` is declared an Int, but `a[f..f].max()` is an ?Int
let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let f = V.first!
let k: Int = a[f..f].max()
print(k + 1)
