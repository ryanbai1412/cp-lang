"""COCI 2011/12 C4 #4 inputs: `gen.py seed size` (n up to `size`) or `gen.py --max`
(n = 3e5): Slavko's n distinct heights, then n distinct new board heights, all in 1..=1e9."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 300000 if MAX else r.randint(2, max(2, min(300000, int(sys.argv[2]))))
top = 10**9 if MAX else r.choice([2 * n, 10**9])
print(n)
print(*r.sample(range(1, top + 1), n))
print(*r.sample(range(1, top + 1), n))
