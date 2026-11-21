import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = rng.randint(2, size * 3 + 3)
q = rng.randint(2, size * 4 + 4)
print(n, q)
for _ in range(q):
    a = rng.randint(1, 3)
    b = rng.randint(1, q if a == 3 else n)
    c = rng.randint(1, n)
    print(a, b, c)
