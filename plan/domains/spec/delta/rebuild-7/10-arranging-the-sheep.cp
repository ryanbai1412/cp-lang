// Codeforces 1520E Arranging The Sheep
// in: t; per test n, then a string of '.'/'*' of length n. out: per test the min moves to line all sheep up.
@!multitest
let n: 1..=1e6 = read()
let s = read<Str of ".*">()
let gap = [i - k for (k, i) in [i for i in s.keys if s[i] == '*'].items]
if gap.is_empty: print(0); return
let mid = gap[gap.len / 2]
print(sum(abs(g - mid) for g in gap))
