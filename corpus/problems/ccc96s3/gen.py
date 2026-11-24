"""gen.py SEED SIZE | --max: CCC 1996 S3. Cases n <= 30, k <= min(n, 7). --max is the single
largest case, n = 20, k = 7 (77520 patterns; n = 30 would print 2e6 lines)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
cases = [(20, 7)] if MAX else []
for _ in range(0 if MAX else r.randint(1, 3)):
    n = r.randint(1, min(30, 2 + 2 * S)); cases.append((n, r.randint(0, min(n, 7))))
print(len(cases))
for n, k in cases: print(n); print(k)
