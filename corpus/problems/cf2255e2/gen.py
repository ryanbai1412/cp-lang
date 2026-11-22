import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
big = size > 100
tests = [(size, size)] if big else [(rng.randint(1, 2 * size), rng.randint(1, 4 * size)) for _ in range(rng.randint(1, 4))]
lim = 10 ** 9 if big or rng.random() < 0.5 else 5
def coord():
    return rng.randrange(2 ** 64) if rng.random() < 0.5 else rng.randint(0, 20)
print(len(tests))
for n, q in tests:
    print(n, q)
    print(*[rng.randint(-lim, lim) for _ in range(n)])
    for _ in range(q):
        t = rng.choices([1, 2, 3, 4], [2, 2, 2, 3])[0]
        if t == 1: print(1, coord(), coord(), rng.randint(-lim, lim))
        elif t == 4: print(4, coord())
        else: print(t, coord(), coord())
