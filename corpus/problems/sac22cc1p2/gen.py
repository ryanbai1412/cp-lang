"""gen.py SEED SIZE | --max: SAC '22 CC1 P2. L, H, W in 1..=1000 and D in 1..=min(L, W).
--max is L = H = W = D = 1000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
lim = min(1000, 1 + 10 * S)
l, h, w = (1000, 1000, 1000) if MAX else (r.randint(1, lim), r.randint(1, lim), r.randint(1, lim))
print(l, h, w)
print(1000 if MAX else r.randint(1, min(l, w)))
