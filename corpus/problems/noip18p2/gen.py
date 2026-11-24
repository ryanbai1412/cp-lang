import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); print(20)
    for _ in range(20): print(100); print(*r.sample(range(1, 25001), 100))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(1, S + 1); print(n)
    print(*r.sample(range(1, 6 * S + 8), n))
