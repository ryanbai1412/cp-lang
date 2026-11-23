import sys
from itertools import product
from math import isqrt, prod


def exhaustive(l, a):
    return min(sum(a) - sum(b) for b in product(*(range(x + 1) for x in a))
               if all(b[i] * b[j] < l for i in range(len(a)) for j in range(i + 1, len(a))))


def by_cap(l, a):
    # Values too big to enumerate (sample 5): wlog b >= 0, and if b_big is the largest final
    # value and c the largest of the rest, every pair is safe iff b_big * c < l and c * c < l.
    # So try every big index and every cap c, giving the rest min(a, c) and big min(a, (l-1)//c).
    best = None
    for big in range(len(a)):
        for c in range(isqrt(l - 1) + 1):
            top = a[big] if c == 0 else min(a[big], (l - 1) // c)
            cost = a[big] - top + sum(x - min(x, c) for i, x in enumerate(a) if i != big)
            best = cost if best is None else min(best, cost)
    return best


it = iter(sys.stdin.read().split())
for _ in range(int(next(it))):
    n, l = int(next(it)), int(next(it))
    a = [int(next(it)) for _ in range(n)]
    print(exhaustive(l, a) if prod(x + 1 for x in a) <= 10**5 else by_cap(l, a))
