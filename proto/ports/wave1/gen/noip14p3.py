import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.randint(1, 2 * S); k = r.randint(0, n - 1) if n > 1 else 0
cols = r.sample(range(1, n), k) if k else []
print(n, m, k)
for _ in range(n): print(r.randint(1, min(3, m)), r.randint(1, min(3, m)))
for c in cols:
    low = r.randint(0, m - 1); print(c, low, r.randint(low + 1, m))
