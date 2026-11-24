// Codeforces 2245 A, tuned: one loop over the prefix and the mirrored suffix,
// as orig.cpp does. `s[0..k].count(c => c == 'R')` on a `[0..n]Char` copies `s` and then
// the slice before counting (a slice of an integer-keyed array is lowered as a
// fresh list, not a view); this spelling reads `s` in place.
@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let s: [0..n]Char = read()
if n < 2 * k:
    print(-1)
    return
print(sum(Int(s[i] != 'R') + Int(s[n - 1 - i] != 'L') for i in 0..k))
