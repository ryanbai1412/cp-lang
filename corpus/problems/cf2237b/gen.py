import sys, random
if sys.argv[1] == "--max":
    # one test at sum n = 2000: each pile only fits the mirrored slot, so every fill bubbles a
    # pile across the whole row (n^2 / 2 swaps)
    n = 2000
    b = [500_000 * j for j in range(1, n + 1)]
    print(1)
    print(n)
    print(*b[::-1])
    print(*b)
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 3); print(n)
    b = sorted(r.randint(1, 3 * n) for _ in range(n))
    a = [r.randint(1, 3 * n + 2) for _ in range(n)]
    print(*a)
    print(*b)
