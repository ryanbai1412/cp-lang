"""cco19p1 (Human Error) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

R C with R * C <= 13, R rows of J/D with at least one of each, then J D in 1..=16.
--max: the 1 x 13 board (the most cells) in a random J/D mix, J = 2 and D = 3, so every
reachable board is searched and the best-move sets are mixed.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
cap = 13 if MAX else max(2, min(13, int(sys.argv[2])))
if MAX:
    r, c = 1, 13
else:
    r, c = rng.choice([(a, b) for a in range(1, 14) for b in range(1, 14) if 2 <= a * b <= cap])
    if rng.random() < 0.5:
        r, c = c, r
cells = [rng.choice("JD") for _ in range(r * c)]
cells[0], cells[-1] = "J", "D"
rng.shuffle(cells)
print(r, c)
print("\n".join("".join(cells[i * c:(i + 1) * c]) for i in range(r)))
print(*((2, 3) if MAX else (rng.randint(1, 16), rng.randint(1, 16))))
