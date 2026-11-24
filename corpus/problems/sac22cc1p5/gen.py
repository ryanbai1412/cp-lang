"""gen.py SEED SIZE | --max: SAC '22 CC1 P5. N, K in 1..=150, C_i in 1..=1e6, H_i in 0..=1e6.
--max is N = K = 150 with full-range values."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 150 if MAX else r.randint(1, min(150, 1 + 2 * S))
k = 150 if MAX else r.randint(1, min(150, 1 + 2 * S))
hi = 10**6 if MAX else r.choice([10, 10**6])
print(n, k)
print(*(r.randint(1, hi) for _ in range(n)))
print(*(r.randint(0, hi) for _ in range(n)))
