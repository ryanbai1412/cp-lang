import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); q = r.randint(1, S + 2); print(n, q)
for _ in range(q):
    l = r.randint(1, n); print(l, r.randint(l, n), r.choice([1, 2, 2, 3, 4, 6, 12, 16, r.randint(1, 16)]))
