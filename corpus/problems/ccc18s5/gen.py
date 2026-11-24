"""gen.py SEED SIZE | --max: CCC 2018 S5. n planets, m cities, p flights, q portals, costs 1..=1e8.
--max is n = m = p = q = 1e5 with costs up to 1e8."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n = m = p = q = 10**5; M = 10**8
else:
    S = int(sys.argv[2])
    n = r.randint(1, S); m = r.randint(1, S); p = r.randint(1, 2 * S); q = r.randint(1, 2 * S)
    M = r.choice([3, 10**8])
print(n, m, p, q)
print("\n".join(f"{r.randint(1, m)} {r.randint(1, m)} {r.randint(1, M)}" for _ in range(p)))
print("\n".join(f"{r.randint(1, n)} {r.randint(1, n)} {r.randint(1, M)}" for _ in range(q)))
