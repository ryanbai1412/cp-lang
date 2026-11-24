"""gen.py SEED SIZE | --max: CCC 2022 S4. n points (3 <= n <= 1e6) on c spots (3 <= c <= 1e6) of a circle.
--max is n = c = 1e6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n = c = 10**6
else:
    S = int(sys.argv[2]); n = r.randint(3, 3 * S + 3); c = r.randint(3, S + 4)
print(n, c)
print(*[r.randrange(c) for _ in range(n)])
