"""gen.py SEED SIZE | --max: NOIP 2018 P1. n in 1..=1e5, d_i in 0..=1e4. --max is n = 1e5 with
alternating 0 and 1e4 (the largest answer)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, min(100_000, 1 + 5 * S))
hi = 10_000 if MAX else r.choice([3, 10_000])
print(n)
print(*((i % 2) * hi for i in range(n)) if MAX else (r.randint(0, hi) for _ in range(n)))
