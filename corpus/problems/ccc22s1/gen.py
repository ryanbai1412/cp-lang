"""gen.py SEED SIZE | --max: CCC 2022 S1 Good Fours and Good Fives. One N with
1 <= N <= 10^6. --max is N = 10^6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
print(10**6 if MAX else r.randint(1, min(10**6, 10 * S * S)))
