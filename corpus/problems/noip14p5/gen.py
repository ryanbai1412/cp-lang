import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); n, m = 10000, 200000; print(n, m)
    for _ in range(m): print(r.randint(1, n), r.randint(1, n))
    print(1, n); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.randint(0, 3 * n)
print(n, m)
for _ in range(m): print(r.randint(1, n), r.randint(1, n))
print(r.randint(1, n), r.randint(1, n))
