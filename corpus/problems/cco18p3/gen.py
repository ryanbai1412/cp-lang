"""gen.py SEED SIZE | --max: CCO 2018 Fun Palace. N <= 1000 rooms, e and every a_i, b_i in
[1, 10000]; SIZE caps N and the values. --max is N = 1000 with values near 10000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, min(1000, 1 + S))
hi = 10000 if MAX else min(10000, r.choice([1, 2, 3, S, 10 * S]) or 1)
lo = 9000 if MAX else 1
print(n)
print(r.randint(lo, hi))
for _ in range(n - 1):
    print(r.randint(lo, hi), r.randint(lo, hi))
