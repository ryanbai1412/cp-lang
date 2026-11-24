"""gen.py SEED SIZE | --max: CCC 2019 S1 Flipper. One line of 1 to 1e6 characters, each H or V.
--max is 1e6 random flips."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10**6 if MAX else r.randint(1, min(10**6, 1 + 4 * S))
print("".join(r.choice("HV") for _ in range(n)))
