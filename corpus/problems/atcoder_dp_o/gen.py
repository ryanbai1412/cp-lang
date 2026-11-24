import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0)
    n = 21
    print(n)
    for _ in range(n): print(*[1] * n)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); p = r.random()
print(n)
for _ in range(n): print(*[int(r.random() < p) for _ in range(n)])
