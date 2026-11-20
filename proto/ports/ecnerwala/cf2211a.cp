// Codeforces 2211 A: print 1 for N == 1, otherwise N copies of 2.
@!multitest
let n: Int = read()
let _a: [0..n]Int = read()
print(List.filled(n, if n == 1: 1 else: 2))
