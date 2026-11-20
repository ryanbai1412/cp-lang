// Codeforces 2245 A: the first k characters must become R and the last k
// must become L, which is impossible when the two ends overlap.
@!multitest
let (n, k): (1..=2e5, 1..=2e5) = read()
let s: Str = read()
if n < 2 * k:
    print(-1)
    return
print(count(s[i] != 'R' for i in 0..k) + count(s[n - 1 - i] != 'L' for i in 0..k))
