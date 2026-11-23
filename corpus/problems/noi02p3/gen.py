import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2]) if len(sys.argv) > 2 else 300
n = r.randint(1, S)
heads = r.choice([2, 2, 3, r.randint(2, max(2, n)), r.randint(2, 300)])
fits = n - heads + 1
k = r.randint(1, fits) if fits >= 1 and r.random() < 0.85 else r.randint(1, n)
print(n, heads, k)
top = r.choice([0, 3, 100, 100_000, 100_000])
shape = r.randint(0, 2)  # path, star-ish, random
for v in range(2, n + 1):
    p = v - 1 if shape == 0 else (1 if shape == 1 and r.random() < 0.5 else r.randint(1, v - 1))
    a, b = (p, v) if r.random() < 0.5 else (v, p)
    print(a, b, r.randint(0, top))
