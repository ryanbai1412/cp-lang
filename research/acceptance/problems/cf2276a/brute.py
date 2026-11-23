import sys
from functools import lru_cache
# most sandwiches (c cheese > s sausage >= 1 each) that use exactly a cheese and b sausage
@lru_cache(maxsize=None)
def most(a, b):
    if a == 0 and b == 0: return 0
    best = -1
    for s in range(1, b + 1):
        for c in range(s + 1, a + 1):
            r = most(a - c, b - s)
            if r >= 0: best = max(best, r + 1)
    return best
sys.setrecursionlimit(10000)
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    a, b = int(next(it)), int(next(it)); print(most(a, b))
