import sys, random
if sys.argv[1:] == ["--max"]:
    # N = 1e6 pirates, K = 1e18, every V[i] shape mixed (random, near i, near half, small).
    rng = random.Random(2016)
    n = 10**6
    v = []
    for i in range(1, n + 1):
        k = rng.randrange(4)
        v.append([rng.randint(1, i), rng.randint(max(1, i - 3), i), (i + 1) // 2, rng.randint(1, max(1, i // 3))][k])
    print(n)
    print(10**18)
    print("\n".join(map(str, v)))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size <= 20:
    n = rng.randint(2, size + 2)
    k = rng.choice([1, 2, 5, 100, 10**18, rng.randint(1, 10**18)])
else:
    n = size
    k = [10**18, 10**18, rng.randint(1, 10**6), 10**18, rng.randint(1, 10**18)][seed % 5]
mode = rng.randint(0, 4) if size <= 20 else seed % 5
v = []
for i in range(1, n + 1):
    if mode == 0:
        v.append(rng.randint(1, i))
    elif mode == 1:
        v.append(rng.randint(max(1, i - 3), i))
    elif mode == 2:
        v.append((i + 1) // 2 + (rng.random() < 0.5 and i > 1))
    elif mode == 3:
        v.append(max(1, i - rng.randint(0, 2)) if rng.random() < 0.9 else rng.randint(1, i))
    else:
        v.append(rng.randint(1, max(1, i // 3)))
print(n)
print(k)
print("\n".join(map(str, v)))
