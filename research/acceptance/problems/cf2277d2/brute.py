# Exhaustive for n <= SMALL. A run of m >= 1 Operation 1s adds 2^m - 1; runs need an
# Operation 2 between them, and Operation 2s commute otherwise. So reaching n with runs
# m_1..m_j and s >= j - 1 Operation 2s means n + s = sum(2^m_i - 1), costing
# sum(m_i) + s. h[j][T] is the least sum(m_i) writing T as j such terms; a solution with
# s >= lim or j >= lim costs >= lim, so the search is exact once the best is below lim.
# Above SMALL (gen's long-range cases near 10^18, sample 5) it uses the easy version's
# O(log n) recursion, checked here against the exhaustive table on every n <= SMALL.
import sys
SMALL = 300
def table(N, lim=16):
    top = N + lim
    terms = [(2 ** m - 1, m) for m in range(1, top.bit_length() + 1) if 2 ** m - 1 <= top]
    INF = float("inf")
    h = [[0] + [INF] * top]
    for _ in range(1, lim):
        prev, cur = h[-1], [INF] * (top + 1)
        for T in range(top + 1):
            for v, m in terms:
                if v <= T and prev[T - v] + m < cur[T]: cur[T] = prev[T - v] + m
        h.append(cur)
    f = [min(s + h[j][n + s] for s in range(lim) for j in range(min(s + 1, lim - 1) + 1)) for n in range(N + 1)]
    return f if max(f) < lim else table(N, 2 * lim)
def rec(n):
    if n < 4: return [0, 1, 3, 2][n]
    k = n.bit_length() - 1
    return min(rec(n - 2 ** k + 2) + k + 1, 2 ** (k + 1) - n + k)
it = iter(sys.stdin.read().split())
qs = [(int(next(it)), int(next(it))) for _ in range(int(next(it)))]
small = table(min(SMALL, max(r for _, r in qs)))
assert all(rec(n) == v for n, v in enumerate(small))
f = lambda n: small[n] if n < len(small) else rec(n)
for l, r in qs: print(sum(f(n) for n in range(l, r + 1)) % 998244353)
