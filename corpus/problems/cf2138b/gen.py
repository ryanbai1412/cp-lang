#!/usr/bin/env python3
# usage: gen.py seed size; size caps the sums of n and of q (the statement allows 5e5 each)
import random, sys
if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = q = 5e5: two interleaved increasing runs (many inversions, few
    # decreasing triples) with a sprinkle of local triples, and random ranges.
    r = random.Random(2138)
    n = q = 500_000
    lo, hi = list(range(1, n // 2 + 1)), list(range(n // 2 + 1, n + 1))
    p, i, j = [], 0, 0
    while i < len(lo) or j < len(hi):
        if j < len(hi) and (i == len(lo) or r.random() < 0.5):
            p.append(hi[j]); j += 1
        else:
            p.append(lo[i]); i += 1
    for _ in range(50):
        k = r.randrange(n - 3)
        p[k:k + 3] = sorted(p[k:k + 3], reverse=True)
    out = ["1", f"{n} {q}", " ".join(map(str, p))]
    for _ in range(q):
        l = r.randint(1, n)
        out.append(f"{l} {r.randint(l, min(n, l + r.choice([5, 1000, n])))}")
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size <= 50:
    T = r.randint(1, 3)
    tests = [(r.randint(1, 10 if size == 0 else 8), r.randint(1, 10 if size == 0 else 50)) for _ in range(T)]
else:
    nb = qb = min(size, 500_000)
    tests = []
    while nb >= 1 and qb >= 1:
        n = nb if r.random() < 0.4 else r.randint(1, nb)
        q = qb if r.random() < 0.4 else r.randint(1, qb)
        tests.append((n, q)); nb -= n; qb -= q
print(len(tests))
for n, q in tests:
    shape = r.choice(["random", "random", "increasing", "decreasing", "zigzag", "blocks"])
    p = list(range(1, n + 1))
    if shape == "random":
        r.shuffle(p)
    elif shape == "decreasing":
        p.reverse()
    elif shape == "zigzag":
        p = [x for i in range(0, n, 2) for x in ([i + 2, i + 1] if i + 2 <= n else [i + 1])]
    elif shape == "blocks":
        b = r.randint(2, 4)
        p = [x for i in range(0, n, b) for x in reversed(range(i + 1, min(n, i + b) + 1))]
    print(n, q)
    print(*p)
    out = []
    for _ in range(q):
        l = r.randint(1, n)
        out.append(f"{l} {r.randint(l, n)}")
    print("\n".join(out))
