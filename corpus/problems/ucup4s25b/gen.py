import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S + 4); print(n)
for _ in range(n):
    print(r.randint(1, 10**9), r.choice("LR"))
q = r.randint(1, 2 * S + 4); print(q)
for _ in range(q):
    l = r.randint(1, n); ri = r.randint(l, n)
    print(l, ri)
