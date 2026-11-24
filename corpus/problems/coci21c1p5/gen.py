"""COCI 2021/22 #1 Volontiranje inputs: `gen.py seed size` (a random permutation, sometimes built
from a few interleaved runs so many longest runs exist), or `gen.py --max` (n = 1e6)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 10**6 if MAX else r.randint(1, S + 1)
if r.randint(0, 1):
    p = list(range(1, n + 1)); r.shuffle(p)
else:
    runs = r.randint(1, 4); p = sorted(range(1, n + 1), key=lambda v: (v % runs, r.random() * (v // runs + 1) ** 0.5))
print(n)
print(*p)
