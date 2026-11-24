import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5, values at 1e9: all of a's mass in the first half and b's in the
    # second, so the deposit stack grows to n / 2 and units travel up to n / 2 rounds
    n = 200_000
    h = n // 2
    a = [10**9] * h + [1] * h
    b = [1] * h + [10**9] * h
    print(1)
    print(n, sum(a) // 2)
    print(*a)
    print(*b)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, size * 4 + 2)
    a = [rng.randint(1, 20) for _ in range(n)]
    b = [rng.randint(1, 20) for _ in range(n)]
    sa, sb = sum(a), sum(b)
    if sa > sb:
        b[rng.randrange(n)] += sa - sb
    k = rng.randint(0, sa)
    print(n, k)
    print(*a)
    print(*b)
