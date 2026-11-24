import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5, values at 1e9: all of a's mass in the first half and b's in the
    # second, so the supply stack grows to n / 2 and units travel up to n / 2 rounds
    n = 200_000
    h = n // 2
    print(1)
    print(n, 0)
    print(*([10**9] * h + [1] * h))
    print(*([1] * h + [10**9] * h))
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
    print(n, 0)
    print(*a)
    print(*b)
