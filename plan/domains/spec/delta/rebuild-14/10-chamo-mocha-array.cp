// Codeforces 1975C Chamo and Mocha's Array
// in: t; per case n, then n values a. out: per case the largest value the whole array can end as
@!multitest
let n = read<2..=1e5>()
let a = read<[n]1..=1e9>()
if n == 2:
    print(min(a[0], a[1]))
else:
    print(a.windows(3).map(w => w.sorted()[1]).max()!)
