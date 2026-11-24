"""COCI 2013/14 C1 Ratar inputs: `gen.py seed size` (n up to `size`, values from a narrow or
the full range) or `gen.py --max` (n = 50, values in -1..=1 so many sums collide)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 50 if MAX else r.randint(1, max(1, min(50, int(sys.argv[2]))))
w = 1 if MAX else r.choice([1, 3, 1000])
print(n)
for _ in range(n): print(*(r.randint(-w, w) for _ in range(n)))
