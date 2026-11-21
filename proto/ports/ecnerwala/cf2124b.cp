// Codeforces 2124 B: prefix-min sum after at most one operation is a[0] + min(a[0], a[1]).
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(a[0] + min(a[0], a[1]))
