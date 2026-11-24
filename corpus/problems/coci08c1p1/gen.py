"""gen.py SEED SIZE | --max: COCI 2008/2009 Contest 1 #1 (Skocimis). Three kangaroo positions
0 < A < B < C < 100. --max puts them at 1 2 99, the longest game."""
import random, sys
if sys.argv[1] == "--max":
    print("1 2 99")
    sys.exit()
r = random.Random(int(sys.argv[1]))
hi = min(99, 3 + 3 * int(sys.argv[2]))
print(*sorted(r.sample(range(1, hi + 1), 3)))
