#!/usr/bin/env python3
# gen.py SEED SIZE: SIZE 0/1 small; SIZE >= 2 sum of m = 2e5 with values up to 2e5.
import random, sys
MAX = sys.argv[1] == "--max"  # sum m = 2e5 in one dense test over 120 values (huge SCCs)
if MAX:
    sys.argv[1:] = ["0", "2"]
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
V = 2 * 10 ** 5

def triples(m, k):
    seen = set()
    while len(seen) < m:
        seen.add(tuple(sorted(r.sample(range(1, k + 1), 3))))
    out = list(seen)
    r.shuffle(out)
    return [r.sample(t, 3) for t in out]

tests = []
if S <= 1:
    for _ in range(4):
        m = r.randint(1, 2 * S + 2)
        tests.append(triples(m, r.randint(max(3, 5 if m > 1 else 3), min(8, m + 3))))
else:
    shape = 1 if MAX else r.randint(0, 3)
    if shape == 0:
        tests.append(triples(V, V))  # sparse over the full value range
    elif shape == 1:
        tests.append(triples(V, 120))  # dense: few values, huge SCCs
    elif shape == 2:
        tests.append([[i, i % V + 1, (i + 1) % V + 1] for i in range(1, V + 1)])  # one long ring
    else:
        left = V
        while left > 0 and len(tests) < 2 * 10 ** 4:
            k = r.randint(5, 12)
            m = min(left, r.randint(1, 30), k * (k - 1) * (k - 2) // 6)
            tests.append(triples(m, k)); left -= m
print(len(tests))
for t in tests:
    print(len(t))
    for a in t: print(*a)
