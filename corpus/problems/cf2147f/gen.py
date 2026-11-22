import random, sys

if "--max" in sys.argv[1:]:
    rng = random.Random(214706)
    cases = [(100_000, 100_000)]
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2])
    rng = random.Random(seed)
    t = rng.randint(1, 3)
    cases = [(rng.randint(2, max(2, 2 * size + 2)), rng.randint(1, max(1, 3 * size))) for _ in range(t)]

print(len(cases))
for n, q in cases:
    print(n, q)
    p = list(range(1, n + 1))
    s = list(range(1, n + 1))
    rng.shuffle(p)
    rng.shuffle(s)
    print(*p)
    print(*s)
    for _ in range(q):
        tp = rng.randint(1, 2)
        i, j = rng.sample(range(1, n + 1), 2)
        print(tp, i, j)
