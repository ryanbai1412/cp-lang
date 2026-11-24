"""noi21p5 (Code) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = q = 1e5).

"n q", the W/E string, then q updates APPEND c / FLIP l r / REVERSE l r on the current length.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
q = size if MAX else rng.randint(1, size)
ew = 0.5 if MAX else rng.choice([0.0, 0.5, 1.0, rng.random()])
print(n, q)
print("".join("E" if rng.random() < ew else "W" for _ in range(n)))
out, length = [], n
for _ in range(q):
    kind = rng.randrange(3) if not MAX else rng.choice([1, 1, 2, 2, 0])
    if kind == 0:
        out.append(f"APPEND {rng.choice('WE')}")
        length += 1
    else:
        l, r = sorted((rng.randint(1, length), rng.randint(1, length)))
        if MAX and rng.random() < 0.5:
            l, r = rng.randint(1, 10), rng.randint(length - 10, length)
        out.append(f"{'FLIP' if kind == 1 else 'REVERSE'} {l} {r}")
print("\n".join(out))
