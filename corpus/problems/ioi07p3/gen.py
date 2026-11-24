"""ioi07p3 (Sails) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N (2 <= N <= 1e5), then N masts "H K" (1 <= H <= 1e5, 1 <= K <= H).
--max: N = 1e5 masts of random height up to 1e5, each carrying about half its height in sails.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, top = 100_000, 100_000
else:
    size = max(2, int(sys.argv[2]))
    n, top = rng.randint(2, min(100_000, size)), rng.randint(1, min(100_000, 2 * size))
print(n)
for _ in range(n):
    h = rng.randint(1, top)
    k = rng.randint(max(1, h // 3), h) if MAX else rng.randint(1, h)
    print(h, k)
