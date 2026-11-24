"""gen.py SEED SIZE | --max: NOIP 2018 P5. One line `n m` with n <= 8 and m <= 1e6 (the subtask
table's bounds). Small sizes keep both sides small, either side the longer; --max is 8 x 1e6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    print(8, 10**6)
else:
    print(r.randint(1, min(8, 1 + S)), r.randint(1, min(10**6, 1 + S * S)))
