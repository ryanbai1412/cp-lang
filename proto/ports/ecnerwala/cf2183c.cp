// Codeforces 2183 C: closed-form maximum reachable positions from K within M steps.
@!multitest
let (n, m_in, k): (Int, Int, Int) = read()
let m = m_in + 1
let l = min(k - 1, n - k)
let r = max(k - 1, n - k)
if l + 2 * r <= m: print(l + r + 1)
elif 3 * l <= m: print(l + (m - l) / 2 + 1)
else: print(2 * m / 3 + 1)
