import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 4 + 2)
    print(n)
    a = list(range(1, n + 1))
    rng.shuffle(a)
    if rng.random() < 0.65:
        b = a[::-1]
        p = list(range(n))
        rng.shuffle(p)
        a = [a[i] for i in p]
        b = [b[i] for i in p]
    else:
        b = list(range(1, n + 1))
        rng.shuffle(b)
    print(*a)
    print(*b)
