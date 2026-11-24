import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0)
    n = 200000
    print(n, 10**12)
    print(*sorted(r.sample(range(1, 10**6 + 1), n)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 2); print(n, r.randint(1, 60))
h = sorted(r.sample(range(1, 40), n)); print(*h)
