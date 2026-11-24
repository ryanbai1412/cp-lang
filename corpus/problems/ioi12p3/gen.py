#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n operations `T c`, `U x`, `P i` with every undo and print
# in range.
import random, sys
MAX = sys.argv[1] == "--max"  # 1e6 operations
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**6 if MAX else r.randint(1, 12)
texts, out = [""], []
for _ in range(n):
    cur = texts[-1]
    k = r.random()
    if cur and k < 0.3:
        out.append(f"P {r.randrange(len(cur))}")
    elif len(texts) > 1 and k < 0.5:
        x = r.randint(1, len(texts) - 1)
        out.append(f"U {x}"); texts.append(texts[-1 - x])
    else:
        c = r.choice("abc" if not MAX else "abcdefghijklmnopqrstuvwxyz")
        out.append(f"T {c}"); texts.append(cur + c if not MAX else "x" * (len(cur) + 1))
print(n)
print("\n".join(out))
