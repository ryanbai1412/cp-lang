// Codeforces 1986C Update Queries
// in: t; per case n m, string s (len n), m indices (1-based), string c (len m). out: per case the smallest string
@!multitest
let (n, m) = read<(1..=1e5, 1..=1e5)>()
var s = read<Str>()
let ind = read<[m]1..=n>()
let c = read<Str>()
let spots = ind.sorted().deduped()
let letters = c.sorted()
for (j, i) in spots.items:
    s[i - 1] = letters[j]
print(s)
