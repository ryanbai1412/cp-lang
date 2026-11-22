import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 2); print(n, r.randint(1, 60))
h = sorted(r.sample(range(1, 40), n)); print(*h)
