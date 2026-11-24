"""gen.py SEED SIZE | --max: CCO 2017 P1. One K in 1..=1e7. --max is K = 1e7."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
print(10 ** 7 if MAX else r.randint(1, min(10 ** 7, 10 ** min(7, 1 + S))))
