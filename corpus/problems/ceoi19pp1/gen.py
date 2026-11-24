"""gen.py seed size: CEOI '19 Practice P1. h, v <= 1500 lines at strictly increasing
coordinates in 0..=2^30. --max: h = v = 1500, both arithmetic progressions with one step,
so every gap repeats and the gap map takes the most hits."""
import random, sys
if sys.argv[1:] == ["--max"]:
    n = m = 1500
    print(n, m); print(*range(0, 700_000 * n, 700_000)); print(*range(5, 5 + 700_000 * m, 700_000))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
n, m = r.randint(1, max(1, min(size, 1500))), r.randint(1, max(1, min(size, 1500)))
hi = r.choice([3 * max(n, m), 2 ** 30])
print(n, m); print(*sorted(r.sample(range(hi + 1), n))); print(*sorted(r.sample(range(hi + 1), m)))
