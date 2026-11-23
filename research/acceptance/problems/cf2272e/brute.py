import sys
from itertools import combinations
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, m, s, u = int(next(it)), int(next(it)), next(it), next(it)
    got = {("A", i): set() for i in range(n) if s[i] == "R"} | {("B", j): set() for j in range(m) if u[j] == "R"}
    for pos in combinations(range(n + m), n):
        pa, i, j, cell = set(pos), 0, 0, 0
        for ts in range(1, n + m + 1):
            if ts - 1 in pa: who, c, idx = "A", s[i], i; i += 1
            else: who, c, idx = "B", u[j], j; j += 1
            if c == "W": cell = ts
            else: got[(who, idx)].add(cell)
    print(*[len(got[("A", i)]) for i in range(n) if s[i] == "R"])
    print(*[len(got[("B", j)]) for j in range(m) if u[j] == "R"])
