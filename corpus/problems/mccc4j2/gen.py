"""gen.py SEED SIZE | --max: Mock CCC '22 C2 J2. 0 <= N, A, B, X_i, Y_i <= 1e5. Small sizes use
small heights, so obstacles often equal A or B and ties are common; N = 0 shows up at size 1.
--max is N = 1e5 with heights over the full range."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(0, min(100_000, 1 + 2 * S))
h = 100_000 if MAX else min(100_000, 2 + S)
a, b = (r.randint(0, h) for _ in range(2))
out = [f"{n} {a} {b}"] + [f"{r.randint(0, h)} {r.randint(0, h)}" for _ in range(n)]
sys.stdout.write("\n".join(out) + "\n")
