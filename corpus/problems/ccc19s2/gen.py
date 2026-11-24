"""gen.py SEED SIZE | --max: CCC 2019 S2. t values 4 <= n <= 1e6, t <= 1000.
--max is t = 1000 values near 1e6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 10**6 if MAX else int(sys.argv[2])
t = 1000 if MAX else r.randint(1, min(1000, S))
hi = min(10**6, max(4, S * S))
ns = [10**6 - i for i in range(t)] if MAX else [r.randint(4, hi) for _ in range(t)]
print(t)
print("\n".join(map(str, ns)))
