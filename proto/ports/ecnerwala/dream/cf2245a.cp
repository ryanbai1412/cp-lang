// Codeforces 2245 A — Prefix R / Suffix L
//
// The first `k` characters must all become `'R'` and the last `k` must all
// become `'L'`. This is impossible iff the prefix and suffix overlap
// (`n < 2 * k`); otherwise the minimum edits equal the number of mismatched
// positions in `s[0..k]` and `s[n - k..n]`.

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let s: [0..n]Char = read()
if n < 2 * k:
    print(-1)
    return
print(2 * k - s[0..k].count('R') - s[n - k..n].count('L'))
