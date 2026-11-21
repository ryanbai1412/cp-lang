// Codeforces 2245 B: after subtracting c, the larger half (rounded up) must
// be taken; each of the remaining values is taken only when positive.
@!multitest
let (n, c): (1..=2e5, Int) = read()
let a: [0..n]Int = read()
let b = a.sorted()
print(sum(max(x - c, 0) for x in b[0..n / 2]) + sum(x - c for x in b[n / 2..n]))
