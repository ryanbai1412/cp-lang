#!/usr/bin/env python3
# gen.py SEED SIZE: SIZE 0/1 small multitests; SIZE >= 2 sum of n = 5e5 (one big test or many).
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size <= 1:
    ns = [r.randint(2, 8 if size == 0 else 40) for _ in range(r.randint(1, 3))]
else:
    total = 5 * 10 ** 5
    shape = r.randint(0, 2)
    ns = [total] if shape == 0 else [total - 2, 2] if shape == 1 else []
    while shape == 2 and total >= 2:
        k = min(total, r.randint(2, 2000))
        if total - k == 1: k += 1
        ns.append(k); total -= k
out = [str(len(ns))]
for n in ns:
    pts = list(range(1, 2 * n + 1))
    mode = r.randint(0, 2) if size >= 2 else 0
    if mode == 1:
        pairs = [(i, 2 * n + 1 - i) for i in range(1, n + 1)]  # all chords nested: no crossings
    elif mode == 2:
        pairs = [(i, i + n) for i in range(1, n + 1)]  # all chords cross each other
    else:
        r.shuffle(pts)
        pairs = [tuple(sorted(pts[2 * i:2 * i + 2])) for i in range(n)]
    r.shuffle(pairs)
    out.append(str(n))
    out.extend("%d %d" % p for p in pairs)
print("\n".join(out))
