"""gen.py SEED SIZE | --max: DWITE '08 R5 #4 (Wiring the Server Room). 5 maps, each 10x10
with 2 <= nodes <= 20 '#'; node counts scale with SIZE. --max is 5 maps of 20 nodes."""
import random, sys


def board(r, n):
    cells = r.sample(range(100), n)
    g = [["."] * 10 for _ in range(10)]
    for c in cells:
        g[c // 10][c % 10] = "#"
    return ["".join(row) for row in g]


if sys.argv[1] == "--max":
    r, counts = random.Random(0), [20] * 5
else:
    r = random.Random(int(sys.argv[1]))
    hi = max(2, min(20, int(sys.argv[2])))
    counts = [r.randint(2, hi) for _ in range(5)]
print("\n".join(line for n in counts for line in board(r, n)))
