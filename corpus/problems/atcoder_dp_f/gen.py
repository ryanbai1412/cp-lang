#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "3000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 3000:
    n = m = 3000
    letters = "ab"
else:
    n, m = r.randint(1, min(size, 3000)), r.randint(1, min(size, 3000))
    letters = "abcdefghijklmnopqrstuvwxyz"[: r.randint(1, 4 if size < 50 else 26)]
print("".join(r.choice(letters) for _ in range(n)))
print("".join(r.choice(letters) for _ in range(m)))
