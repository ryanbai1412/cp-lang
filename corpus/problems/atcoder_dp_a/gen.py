"""gen.py SEED SIZE | --max: AtCoder DP A (Frog 1). n stones (2 <= n <= 1e5), heights 1..1e4.
Small cases use small heights so jump costs tie. --max is n = 1e5 with random heights."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(2, 2 + S)
hi = 10_000 if MAX else r.choice([3, 10_000])
print(n)
print(*(r.randint(1, hi) for _ in range(n)))
