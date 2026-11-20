// Codeforces 2245 B: after subtracting c, the larger half (rounded up) must
// be taken; each of the remaining values is taken only when positive.
@!multitest
let (n, c): (1..=2e5, Int) = read()
let a: [0..n]Int = read()
let gain = sorted([x - c for x in a])
print(sum(max(gain[i], 0) for i in 0..n / 2) + sum(gain[i] for i in n / 2..n))
