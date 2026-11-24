"""gen.py SEED SIZE | --max: AtCoder DP C (Vacation). n days (1 <= n <= 1e5), three activities worth
1..1e4 each. --max is n = 1e5 with random values."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, 1 + S)
hi = 10_000 if MAX else r.choice([3, 10_000])
print(n)
for _ in range(n):
    print(*(r.randint(1, hi) for _ in range(3)))
