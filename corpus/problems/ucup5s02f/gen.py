#!/usr/bin/env python3
# gen.py SEED SIZE: two lists of sorted disjoint intervals in 0..=M and query periods up to 2M.
import random, sys

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
M = 6 * (S + 2) * r.randint(1, 4)


def intervals():
    cuts = sorted(r.sample(range(M + 1), 2 * r.randint(1, min(S, M // 2))))
    if r.random() < 0.5:
        cuts[0], cuts[-1] = 0, M
    return [(cuts[2 * i], cuts[2 * i + 1]) for i in range(len(cuts) // 2)]


p0, p1 = intervals(), intervals()
q = r.randint(1, 2 * S + 2)
print(len(p0), len(p1))
print(" ".join(f"{l} {h}" for l, h in p0))
print(" ".join(f"{l} {h}" for l, h in p1))
print(q)
print(" ".join(str(r.randint(1, 2 * M)) for _ in range(q)))
