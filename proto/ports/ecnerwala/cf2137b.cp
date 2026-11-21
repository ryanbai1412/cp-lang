// Codeforces 2137 B: complement each element p[i] to n + 1 - p[i].
@!multitest
let n: Int = read()
let p: [0..n]Int = read()
print([n + 1 - x for x in p])
