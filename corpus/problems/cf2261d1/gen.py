import sys, random
if sys.argv[1] == "--max":  # one case n = 2e5 (sum n), m = 0
    print(1); print(200000, 0); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t): print(r.randint(1, 3 * S), 0)
