"""gen.py SEED SIZE | --max: NOIP 2008 P1. One lowercase word of length 1..=100. Small sizes use
few letters so counts repeat; --max is 100 letters."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100 if MAX else r.randint(1, min(100, 2 + 3 * S))
k = 26 if MAX else r.randint(1, 26)
print("".join(r.choice("abcdefghijklmnopqrstuvwxyz"[:k]) for _ in range(n)))
