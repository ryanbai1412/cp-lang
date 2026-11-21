import random, sys

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    x = rng.randint(1, max(1, 3 * size + 2))
    y = rng.randint(1, max(1, 3 * size + 2))
    k = rng.randint(1, x + y - 1)
    a = "".join(rng.choice("01") for _ in range(x + y))
    print(x, y, k)
    print(a)
