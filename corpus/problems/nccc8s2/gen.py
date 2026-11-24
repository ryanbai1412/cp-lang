"""nccc8s2 generator: a lowercase string. --max: length 1e5 over all 26 letters."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 10**5 if big else r.randint(1, 1 + size * 4)
abc = "abcdefghijklmnopqrstuvwxyz"[: 26 if big else r.randint(1, 26)]
print("".join(r.choice(abc) for _ in range(n)))
