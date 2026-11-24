"""mccc3j3 generator: |S| |T|, then uppercase S and T. --max: |S| = |T| = 1e6."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 10**6 if big else r.randint(1, 1 + size * 3)
m = 10**6 if big else r.randint(1, 1 + size * 3)
abc = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"[: 26 if big else r.randint(1, 26)]
print(n, m)
print("".join(r.choice(abc) for _ in range(n)))
print("".join(r.choice(abc) for _ in range(m)))
