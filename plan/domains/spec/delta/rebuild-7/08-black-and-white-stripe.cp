// Codeforces 1690D Black and White Stripe
// in: t; per test n k, then a string of 'W'/'B' of length n. out: per test the min cells to recolour for k consecutive 'B'.
@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let s = read<Str of "WB">()
let white = Prefix([if c == 'W': 1 else: 0 for c in s])
print((0..=n - k).map(l => white.fold(l..l + k)).min()!)
