"""Cheerio 1 S1 inputs: `gen.py seed size` (up to `size` lights) or `gen.py --max` (n = 2e5
lights spread to D = 1e9, cycles up to 1e9)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 200000 if MAX else r.randint(1, max(1, int(sys.argv[2])))
C = 10**9 if MAX or r.random() < .3 else r.randint(n + 1, 4 * n + 10)
ps = sorted(r.sample(range(1, C), n))
hi = 10**9 if MAX else r.choice([3, 20, 10**9])
print(n, r.randint(ps[-1], C))
for p in ps: print(p, r.randint(1, hi), r.randint(1, hi))
