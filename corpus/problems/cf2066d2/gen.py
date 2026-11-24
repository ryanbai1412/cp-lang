#!/usr/bin/env python3
import random, sys

if sys.argv[1] == "--max":
    # one test at sum m = 1e4: n = c = 100, m = n c; floors launch c each in rising order
    # (the only valid order at m = n c), 70% of the entries erased
    r = random.Random(1)
    n = c = 100
    a = [f if r.random() < 0.3 else 0 for f in range(1, n + 1) for _ in range(c)]
    print(1)
    print(n, c, n * c)
    print(*a)
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
T = 5 if S == 0 else 4
print(T)
for _ in range(T):
    hi = 5 if S == 0 else min(25, S + 6)
    n = r.randint(1, hi)
    c = r.randint(1, hi)
    m = r.randint(c, min(n * c, 80 if S == 0 else 200))
    # Generate a valid sequence by simulating, then erase random entries (and sometimes corrupt)
    mode = r.choice(["valid", "valid", "valid", "zeros", "corrupt"])
    if mode == "zeros":
        a = [0] * m
    else:
        seen = [0] * (n + 1)
        seq = []
        ok = True
        for _ in range(m):
            cands = [f for f in range(1, n + 1) if seen[f] < c]
            if not cands:
                ok = False
                break
            f = r.choice(cands)
            seq.append(f)
            for g in range(1, f + 1):
                seen[g] += 1
        if not ok or seen[n] < c:
            a = [0] * m
        else:
            p_keep = r.uniform(0.0, 0.7)
            a = [x if r.random() < p_keep else 0 for x in seq]
            if mode == "corrupt":
                a[r.randrange(m)] = r.randint(1, n)
    print(n, c, m)
    print(*a)
