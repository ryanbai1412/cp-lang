"""gen.py SEED SIZE | --max: NOIP 2008 P2. One integer n <= 24 matchsticks; --max is n = 24."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
print(24 if MAX else r.randint(1, 24))
