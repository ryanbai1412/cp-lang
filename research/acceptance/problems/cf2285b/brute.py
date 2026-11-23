import sys
from itertools import combinations
def div(s):
    n = len(s); got = {s}
    for l in range(n):
        for r in range(l, n):
            for x in "01": got.add(s[:l] + x * (r - l + 1) + s[r + 1:])
    return len(got)
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, k, s = int(next(it)), int(next(it)), next(it)
    best = 0
    for c in range(k + 1):
        for idx in combinations(range(n), c):
            u = list(s)
            for i in idx: u[i] = "1" if u[i] == "0" else "0"
            best = max(best, div("".join(u)))
    print(best)
