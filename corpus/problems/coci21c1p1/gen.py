"""gen.py seed size: COCI '21 C1 #1. n <= 100 sprays (t, a, b), t strictly increasing;
players 1-4 and 5-8 are the two teams, and b is always on a's opposing team."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 100 sprays a few seconds apart, so most earn the 50 bonus
    g = random.Random(1)
    t, out = 0, ["100"]
    for _ in range(100):
        t += g.randint(1, 3)
        a = g.randint(1, 8)
        out.append(f"{t} {a} {g.randint(5, 8) if a <= 4 else g.randint(1, 4)}")
    print("\n".join(out))
    sys.exit()


seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(1, max(1, min(100, size)))
gap = rng.choice([3, 12, 30])
t, out = 0, [str(n)]
for _ in range(n):
    t += rng.randint(1, gap)
    a = rng.randint(1, 8)
    b = rng.randint(5, 8) if a <= 4 else rng.randint(1, 4)
    out.append(f"{t} {a} {b}")
print("\n".join(out))
