import sys, random

if sys.argv[1] == "--max":
    # one test near sum n = 2e5: a treasure plus 49999 operations spread over the whole
    # coordinate range, so the lit bulbs span |x|, |y| <= 1e8
    rng = random.Random(1)
    L = 10**8 - 1
    pts = {(rng.randint(-L, L), rng.randint(-L, L))}
    for _ in range(49_999):
        x, y = rng.randint(-L, L - 1), rng.randint(-L + 1, L - 1)
        for p in [(x, y), (x, y + 1), (x + 1, y - 1), (x + 1, y)]:
            pts ^= {p}
    pts = list(pts)
    rng.shuffle(pts)
    print(1)
    print(len(pts))
    print("\n".join(f"{x} {y}" for x, y in pts))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    sx, sy = rng.randint(-20, 20), rng.randint(-20, 20)
    pts = {(sx, sy)}
    ops = rng.randint(0, size * 3)
    for _ in range(ops):
        x, y = rng.randint(-15, 15), rng.randint(-15, 15)
        for p in [(x, y), (x, y + 1), (x + 1, y - 1), (x + 1, y)]:
            if p in pts: pts.remove(p)
            else: pts.add(p)
    pts = list(pts)
    rng.shuffle(pts)
    print(len(pts))
    for x, y in pts:
        print(x, y)
