"""noi21p1 (Light and Heavy Edges) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

T, then per test "n m", n - 1 edges, and m operations "1 a b" or "2 a b". --max: T = 3 tests
of n = m = 1e5 (a long path, a complete binary tree, a random tree), operations between far
vertices, half updates and half queries.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
t = 3 if MAX else rng.randint(1, 3)
out = [str(t)]
for kind in range(t):
    n = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
    m = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
    shape = kind if MAX else rng.randrange(3)
    lab = list(range(1, n + 1))
    rng.shuffle(lab)
    out.append(f"{n} {m}")
    for i in range(1, n):
        p = i - 1 if shape == 0 else (i - 1) // 2 if shape == 1 else rng.randrange(i)
        out.append(f"{lab[p]} {lab[i]}" if rng.random() < 0.5 else f"{lab[i]} {lab[p]}")
    for _ in range(m):
        if MAX and shape != 2:
            a, b = lab[rng.randrange(n // 10 + 1)], lab[n - 1 - rng.randrange(n // 10 + 1)]
        else:
            a, b = rng.randint(1, n), rng.randint(1, n)
        out.append(f"{rng.randint(1, 2)} {a} {b}")
print("\n".join(out))
