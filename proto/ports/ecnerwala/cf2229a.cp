// Codeforces 2229 A: ceiling of half the array span (max - min).
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(ceil_div(a.max()! - a.min()!, 2))
