import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 5 + 3)
    q = 1
    w = 1
    while w < n:
        q += 1
        w *= 3
    fake = rng.randint(1, n)
    hide = rng.randint(-1, q - 1)
    print(n, fake, hide)
