"""gen.py SEED SIZE | --max: WC 2022 P3's hidden data: T = 1000 games (the statement fixes T and
the 8869-word dictionary, so SIZE changes nothing) and each game's answer, a dictionary index
drawn independently and uniformly. --max is the same shape from seed 0."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
assert MAX or int(sys.argv[2]) >= 1
print(1000)
print(*(r.randrange(8869) for _ in range(1000)))
