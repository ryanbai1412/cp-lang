"""ioi18p2 generator: `h w q`, the seat (row, column) of each contestant, then q swaps `a b`."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 1000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 4)
h = size if big else r.randint(1, max(1, size))
w = size if big else r.randint(1, max(1, size))
q = 50_000 if big else r.randint(0, 3 * size)
cells = [(y, x) for y in range(h) for x in range(w)]
if not big and r.random() < 0.5:
    cells.sort(key=lambda s: (max(s), s))  # many rectangle prefixes to start
else:
    r.shuffle(cells)
print(h, w, q)
print(" ".join(f"{y} {x}" for y, x in cells))
for _ in range(q):
    a = r.randrange(h * w)
    b = r.randrange(h * w)
    print(a, b)
