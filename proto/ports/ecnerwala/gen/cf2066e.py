import random, sys

if "--max" in sys.argv[1:]:
    rng = random.Random(206605)
    n, q = 200_000, 200_000
    cur = [2000 + i * 1900 for i in range(500)] + [rng.randint(1, 20_000) for _ in range(n - 500)]
    mx = 20_000
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2])
    rng = random.Random(seed)
    n = rng.randint(1, max(1, 3 * size + 2))
    q = rng.randint(1, max(1, 4 * size + 2))
    mx = rng.choice([20, 100, 2000, 20000])
    cur = [rng.randint(1, mx) for _ in range(n)]

print(n, q)
print(*cur)
for _ in range(q):
    if len(cur) > 1 and rng.randint(0, 2) == 0:
        idx = rng.randrange(len(cur))
        v = cur[idx]
        cur[idx] = cur[-1]
        cur.pop()
        print("-", v)
    else:
        v = rng.randint(1, mx)
        cur.append(v)
        print("+", v)
