#!/usr/bin/env python3
# gen.py SEED SIZE: two lists of sorted disjoint intervals in 1..=M and query periods up to 2M.
# --max: n = m = 50 intervals up to 1e18, the main ones covering all but 49 narrow gaps, the shared
# ones with gaps from 1e3 to 1e17 wide, and q = 100 targets picked for the longest refinement chains.
import random, sys

def steps(T, p0, p1):
    """The refinements of k the obvious search makes for target T."""
    def covers(k, v, p):
        i = 0
        for l, h in p:
            if i >= k: return True
            lo, hi = 6 * k * l - T * v, 6 * k * h - T * v
            if lo > 6 * T * i: return False
            if hi >= 6 * T * i: i = hi // (6 * T) + 1
        return i >= k
    k, n = 1, 0
    while 6 * k <= T and covers(k, 6, p0):
        n += 1
        if not covers(k, 1, p1) or not covers(k, 5, p1): k *= 6
        elif not covers(k, 2, p1) or not covers(k, 4, p1): k *= 3
        elif not covers(k, 3, p1): k *= 2
        else: break
    return n


if sys.argv[1] == "--max":
    r, top = random.Random(0), 10**18
    gaps = sorted(r.sample(range(2, top - 10**6), 49))
    p0 = [(1 if i == 0 else gaps[i - 1] + r.randint(1, 1000), top if i == 49 else gaps[i] - 1) for i in range(50)]
    cuts = sorted(r.sample(range(2, top // 2), 49))
    p1, lo = [], 1
    for c in cuts:
        p1.append((lo, max(lo, c)))
        lo = max(lo, c) + 1 + int(10 ** r.uniform(3, 17))
    p1.append((lo, top))
    ts = sorted((r.randint(6, top) for _ in range(3000)), key=lambda t: -steps(t, p0, p1))[:100]
    print(50, 50)
    print(" ".join(f"{l} {h}" for l, h in p0))
    print(" ".join(f"{l} {h}" for l, h in p1))
    print(100)
    print(" ".join(map(str, ts)))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
M = 6 * (S + 2) * r.randint(1, 4)


def intervals():
    cuts = sorted(r.sample(range(1, M + 1), 2 * r.randint(1, min(S, 50, M // 2))))
    if r.random() < 0.5:
        cuts[0], cuts[-1] = 1, M
    return [(cuts[2 * i], cuts[2 * i + 1]) for i in range(len(cuts) // 2)]


p0, p1 = intervals(), intervals()
q = r.randint(1, min(100, 2 * S + 2))
print(len(p0), len(p1))
print(" ".join(f"{l} {h}" for l, h in p0))
print(" ".join(f"{l} {h}" for l, h in p1))
print(q)
print(" ".join(str(r.randint(1, 2 * M)) for _ in range(q)))
