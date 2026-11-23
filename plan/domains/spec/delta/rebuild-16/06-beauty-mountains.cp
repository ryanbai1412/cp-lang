// Codeforces 1982D Beauty of the Mountains: can k x k additions equalize the two kinds' height sums?
// In: t; per test `n m k`, n rows of heights, n binary strings of types. Out: YES/NO per test.
@!multitest
let n, m: 1..=500 = read()
let k = read<1..=500>()
domain R = 0..n
domain C = 0..m
let h = read<[R, C]0..=1e9>()
let kind = read<[R, C]Char>()

// Adding x to a window changes (snowy - plain) by x * (its snowy count - its plain count).
let sign = [R, C] (r, c) => if kind[r, c] == '0': 1 else: -1
let gap = sum(sign[p] * h[p] for p in (R, C))
let pre = Prefix(sign)
var g = 0
for i in R.cuts, j in C.cuts:
    if let i2 = R.cuts.at(Int(i) + k), let j2 = C.cuts.at(Int(j) + k):
        g = gcd(g, pre.fold(i..i2, j..j2))
print(yes(gap == 0 or (g != 0 and gap % g == 0)))
