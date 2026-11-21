import sys, random
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
