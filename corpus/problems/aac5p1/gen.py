"""gen.py SEED SIZE | --max: AAC 5 P1. n <= 2e5 batches of 1..1e9 cookies. Small cases use small
counts. --max is n = 2e5 random batches up to 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(1, 1 + S)
hi = 10**9 if MAX or r.random() < 0.5 else r.choice([1, 2, 9])
print(n)
print(*(r.randint(1, hi) for _ in range(n)))
