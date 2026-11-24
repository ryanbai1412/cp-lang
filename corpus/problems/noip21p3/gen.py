import sys, random
if sys.argv[1] == "--max":  # n = 1e4, a sorted over all of 1..600 (every gap value present)
    rng = random.Random(0); n = 10**4
    a = sorted(list(range(1, 601)) + [rng.randint(1, 600) for _ in range(n - 600)])
    print(n); print(*a); sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2]) if len(sys.argv) > 2 else 1
n = r.randint(1, 4 + 4 * S)
a = sorted(r.randint(1, min(600, 20 * S)) for _ in range(n))
print(n)
print(*a)
