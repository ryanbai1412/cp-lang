import sys, random
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
