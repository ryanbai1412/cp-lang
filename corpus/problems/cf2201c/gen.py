#!/usr/bin/env python3
# usage: gen.py seed size; size caps the sum of n (the statement allows 3e5)
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n = 3e5: a random regular sequence leaning open, so the balance often
    # drops back under 2 and the counts reset
    g = random.Random(1)
    half = 150_000
    out, op, cl = [], 0, 0
    for _ in range(2 * half):
        if op == half or (op > cl and g.random() >= 0.55):
            out.append(")")
            cl += 1
        else:
            out.append("(")
            op += 1
    print(1)
    print(2 * half)
    print("".join(out))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
budget = max(2, min(int(sys.argv[2]) if len(sys.argv) > 2 else 20, 300_000)) // 2
halves = []
while budget >= 1 and len(halves) < 10_000:
    h = budget if rng.random() < 0.3 else rng.randint(1, budget)
    halves.append(h)
    budget -= h
print(len(halves))
for half in halves:
    shape = rng.choice(["random", "nested", "flat", "deep"])
    if shape == "nested":
        s = "(" * half + ")" * half
    elif shape == "flat":
        s = "()" * half
    else:
        out, op, cl = [], 0, 0
        p = 0.5 if shape == "random" else 0.8
        for _ in range(2 * half):
            if op == half or (op > cl and rng.random() >= p):
                out.append(")"); cl += 1
            else:
                out.append("("); op += 1
        s = "".join(out)
    print(2 * half)
    print(s)
