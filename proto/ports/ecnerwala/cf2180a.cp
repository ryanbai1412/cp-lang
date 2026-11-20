// Codeforces 2180 A: largest value < L congruent to A modulo gcd(L, B).
@!multitest
let (l, a, b): (Int, Int, Int) = read()
let g = gcd(l, b)
print(l - g + a % g)
