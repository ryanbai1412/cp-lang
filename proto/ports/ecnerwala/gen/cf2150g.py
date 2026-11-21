import random, sys

if "--max" in sys.argv[1:]:
    print(1)
    print(5000, 5000, 5000)
    print("0" * 10000)
    sys.exit(0)
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2])
    rng = random.Random(seed)
    t = rng.randint(1, 5)
    cases = []
    for _ in range(t):
        x = rng.randint(1, max(1, 3 * size + 2))
        y = rng.randint(1, max(1, 3 * size + 2))
        k = rng.randint(1, x + y - 1)
        cases.append((x, y, k))

print(len(cases))
for x, y, k in cases:
    a = "".join(rng.choice("01") for _ in range(x + y))
    print(x, y, k)
    print(a)
