"""gen.py SEED SIZE | --max: AtCoder DP B (Frog 2). n stones (2 <= n <= 1e5), jumps up to k (1 <= k <= 100),
heights 1..1e4. --max is n = 1e5, k = 100: the n * k transitions."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(2, 2 + S)
k = 100 if MAX else r.randint(1, min(100, 1 + S))
hi = 10_000 if MAX else r.choice([3, 10_000])
print(n, k)
print(*(r.randint(1, hi) for _ in range(n)))
