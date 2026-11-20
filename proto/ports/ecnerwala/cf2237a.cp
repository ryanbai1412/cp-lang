// Codeforces 2237 A: sum of prefix minima of the array.
@!multitest
let n: Int = read()
var a: [0..n]Int = read()
for i in 1..n: a[i] min= a[i - 1]
print(sum(a))
