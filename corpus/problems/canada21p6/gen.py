"""gen.py SEED SIZE | --max: Canada 2021 P6 3D Rage. n distinct black holes and q blocks
(1 <= n, q <= 500000), coordinates in [0, 6e7]. Small sizes use a small cube so blocks often
dominate a hole and often none (-1); --max is n = q = 500000 over the full cube."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 500_000 if MAX else r.randint(1, min(500_000, 1 + S))
q = 500_000 if MAX else r.randint(1, min(500_000, 1 + S))
c = 60_000_000 if MAX or r.random() < 0.3 else max(2, int(2 * n ** (1 / 3)) + 1)
holes = set()
while len(holes) < n:
    holes.add((r.randint(0, c), r.randint(0, c), r.randint(0, c)))
holes = list(holes)
r.shuffle(holes)
out = [f"{n} {q}"]
out += [f"{x} {y} {z}" for x, y, z in holes]
out += [f"{r.randint(0, c)} {r.randint(0, c)} {r.randint(0, c)}" for _ in range(q)]
print("\n".join(out))
