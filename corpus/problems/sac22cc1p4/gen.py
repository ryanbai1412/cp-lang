"""gen.py SEED SIZE | --max: SAC '22 CC1 P4. N <= 1e5 values A_i in 1..=100. Small sizes draw
from few small values so sums match often; --max is N = 1e5 with small values early and their
triple sums late, so the count is near its largest."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, min(100_000, 1 + 5 * S))
if MAX:
    a = [r.randint(1, 4) for _ in range(n // 2)] + [r.randint(3, 12) for _ in range(n - n // 2)]
else:
    hi = r.choice([3, 10, 100])
    a = [r.randint(1, hi) for _ in range(n)]
print(n)
print(*a)
