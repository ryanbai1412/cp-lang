import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = r.randint(1, S); t = r.randint(1, S); p = r.randint(1, n)
d = r.choice([0.1, 0.5, 0.9, r.random()])
print(n, t, p)
for _ in range(n): print(*[int(r.random() < d) for _ in range(t)])
