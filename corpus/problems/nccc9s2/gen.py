"""nccc9s2 generator: n, then a shuffle of n I's and n U's. --max: n = 1e6."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 10**6 if big else r.randint(1, 1 + size * 2)
s = list("I" * n + "U" * n)
r.shuffle(s)
print(n)
print("".join(s))
