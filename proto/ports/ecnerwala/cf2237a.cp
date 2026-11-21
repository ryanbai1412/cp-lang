// Codeforces 2237 A: sum of prefix minima of the array.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(scan(a, min).sum())
