"""gen.py SEED SIZE | --max: Boolean. Some `not`s, space-separated, then True or False, the line
under 1000 characters. --max is 248 nots and False, 997 characters."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
k = 248 if MAX else r.randint(0, min(248, S))
print(" ".join(["not"] * k + ["False" if MAX else r.choice(["True", "False"])]))
