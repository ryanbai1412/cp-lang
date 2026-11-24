"""gen.py SEED SIZE: CCC 2022 J2 Fergusonball Ratings. N >= 1 players, each with points greater
than fouls, both non-negative. Ratings land on both sides of 40, and sometimes every player is
above it (a gold team). The statement gives no upper bound on N or the counts, so there is no --max."""
import random, sys
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
n = r.randint(1, 1 + 2 * S)
gold = r.random() < 0.3
print(n)
for _ in range(n):
    while True:
        p = r.randint(1, 8 + 2 * S + (10 if gold else 0))
        f = r.randint(0, p - 1)
        if not gold or 5 * p - 3 * f > 40:
            break
    print(p)
    print(f)
