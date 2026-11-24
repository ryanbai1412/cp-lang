"""ioi16p4 generator: a row of X/_/. that some painting fits, k, then the k clues."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 200_000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 10)
n = size if big else r.randint(1, max(1, size))
if big:
    step = n // 100
    paint = ["X" if i % step < step // 2 else "_" for i in range(100 * step)] + ["_"] * (n - 100 * step)
else:
    while True:
        density = r.random()
        paint = ["X" if r.random() < density else "_" for _ in range(n)]
        if any(c == "X" for c in paint) and "".join(paint).count("_X") + (paint[0] == "X") <= 100:
            break
clues = [len(b) for b in "".join(paint).split("_") if b]
known = 0.0 if big else r.choice([0.0, 0.2, 0.6, 1.0])
print("".join(c if r.random() < known else "." for c in paint))
print(len(clues))
print(*clues)
