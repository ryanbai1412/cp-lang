// Codeforces 2183 C: closed-form maximum reachable positions from K within M steps.
@!multitest
let (n, m_in, k): (Int, Int, Int) = read()
let m = m_in + 1
let (l, r) = (min(k - 1, n - k), max(k - 1, n - k))
let ans = if l + 2 * r <= m: l + r + 1
elif 3 * l <= m: l + (m - l) / 2 + 1
else: 2 * m / 3 + 1
print(ans)
