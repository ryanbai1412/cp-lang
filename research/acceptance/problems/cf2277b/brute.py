import sys
from functools import lru_cache
from itertools import combinations
@lru_cache(maxsize=None)
def good(b):
    if b and all(x == 0 for x in b): return True
    for l in range(len(b)):
        for r in range(l, len(b)):
            if b[r] == 0: break
            dec = b[:l] + tuple(x - 1 for x in b[l:r + 1]) + b[r + 1:]
            for k in range(l, r + 1):
                if good(dec[:k] + dec[k + 1:]): return True
    return False
def best(a):
    for m in range(len(a), 0, -1):
        if any(good(c) for c in combinations(a, m)): return m
    return 0
it = iter(sys.stdin.read().split())
for _ in range(int(next(it))):
    n = int(next(it)); print(best(tuple(int(next(it)) for _ in range(n))))
