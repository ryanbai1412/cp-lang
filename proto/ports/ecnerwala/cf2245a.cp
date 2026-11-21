// Codeforces 2245 A: the first k characters must become R and the last k
// must become L, which is impossible when the two ends overlap.
@!multitest
let (n, k): (1..=2e5, 1..=2e5) = read()
let s: Str = read()
if n < 2 * k:
    print(-1)
    return
print(2 * k - s[0..k].count('R') - s[n - k..n].count('L'))
