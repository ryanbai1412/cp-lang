import sys
from functools import lru_cache
@lru_cache(None)
def wins(o, e, need):  # the player to move must take a card of parity `need` (0 even, 1 odd)
    if need == 1: return o > 0 and not wins(o - 1, e, 0)
    return e > 0 and not wins(o, e - 1, 1)
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    x = int(next(it)); ds = [d for d in range(1, int(x ** 0.5) + 1) if x % d == 0]
    ds = set(ds) | {x // d for d in ds}
    o = sum(d % 2 for d in ds); e = len(ds) - o
    print("Yes" if (o and not wins(o - 1, e, 0)) or (e and not wins(o, e - 1, 1)) else "No")
