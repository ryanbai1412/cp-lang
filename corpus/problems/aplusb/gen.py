"""gen.py SEED SIZE | --max: A Plus B. N <= 1e5 pairs, each |x| < 1e9. --max is N = 1e5 with
values at the bounds, so sums reach about +-2e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, min(100_000, 1 + S))
hi = 10**9 - 1 if MAX or r.random() < 0.5 else 10
print(n)
for _ in range(n):
    if MAX:
        s = r.choice([1, -1])
        print(s * r.randint(hi - 1000, hi), s * r.randint(hi - 1000, hi))
    else:
        print(r.randint(-hi, hi), r.randint(-hi, hi))
