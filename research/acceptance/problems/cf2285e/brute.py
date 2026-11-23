import sys
from fractions import Fraction
def fixes(b):
    L = len(b)
    if L <= 2: return 0
    best = L - 1
    for i in range(L):
        for j in range(i + 1, L):
            d = Fraction(b[j] - b[i], j - i)
            if d.denominator != 1: continue
            best = min(best, sum(1 for x in range(L) if b[x] != b[i] + (x - i) * d))
    return best
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, k = int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]
    print(max(r - l for l in range(n) for r in range(l + 1, n + 1) if fixes(a[l:r]) <= k))
