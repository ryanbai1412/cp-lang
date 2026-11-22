import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 2 + 2)
    m = rng.randint(0, min(n * n, size * 4))
    pairs = set()
    while len(pairs) < m:
        pairs.add((rng.randint(1, n), rng.randint(1, n)))
    print(n, len(pairs))
    for i, x in sorted(pairs):
        print(i, x)
