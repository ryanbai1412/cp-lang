"""gen.py SEED SIZE | --max: AAC 4 P1. A string of n <= 1e6 characters, m < n of them 0s and the rest
uppercase letters (at least one letter), and each 0's cheer value 1..1e9. Small cases use small values
so 0s tie. --max is n = 1e6 with m = 5e5 zeros, all with cheer value 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1_000_000 if MAX else r.randint(2, 2 + S)
m = n // 2 if MAX else r.randint(1, n - 1)
pos = set(r.sample(range(n), m))
s = "".join("0" if i in pos else r.choice("ABCDEFGHIJKLMNOPQRSTUVWXYZ" if MAX or r.random() < 0.5 else "AB") for i in range(n))
hi = 10**9 if MAX or r.random() < 0.3 else 3
print(n, m)
print(s)
print(*(10**9 if MAX else r.randint(1, hi) for _ in range(m)))
