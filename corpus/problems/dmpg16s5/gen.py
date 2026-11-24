import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 1e5, k = n / 2, tastes over the full [-1e8, 1e8] with prefix sums going up and down,
    # so the window-minimum deque is rebuilt often
    g = random.Random(1)
    n = 10**5
    t = [g.randint(-10**8, 10**8) for _ in range(n)]
    t[0] = 10**8
    print(n, n // 2)
    print(*t)
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# tastes in [-1e8, 1e8], at least one positive
n = r.randint(1, 2 * S); print(n, r.randint(1, n)); M = r.choice([5, 10**8])
t = [r.randint(-M, M) for _ in range(n)]
t[r.randrange(n)] = r.randint(1, M)
print(*t)
