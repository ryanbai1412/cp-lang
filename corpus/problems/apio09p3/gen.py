import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.randint(1, 2 * S + 1); print(n, m)
for _ in range(m): print(r.randint(1, n), r.randint(1, n))
for _ in range(n): print(r.randint(0, 4000))
start = r.randint(1, n); p = r.randint(1, n); print(start, p)
print(*([start] + [r.randint(1, n) for _ in range(p - 1)]))
