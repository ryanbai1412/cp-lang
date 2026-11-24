import sys, random
if sys.argv[1] == "--max":  # N = M = 5000, Q = 5e4, a = 300, b = c = 1e8, d = 1e8, x0 < d
    r = random.Random(0); n = m = 5000; q = 50000
    out = [f"{r.randrange(10**8)} 300 {10**8} {10**8} {10**8}", f"{n} {m} {q}"]
    out += [f"{r.randint(1, n * m)} {r.randint(1, n * m)}" for _ in range(q)]
    print("\n".join(out)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
d = r.choice([1, 2, r.randint(1, 100), r.randint(1, 10**8), 10**8])
x0 = r.randint(0, d - 1)
a = r.choice([0, r.randint(0, 300), 300])
b = r.choice([0, r.randint(0, 10**8), 10**8])
c = r.choice([0, r.randint(0, 10**8), 10**8])
n = r.randint(1, S); m = r.randint(1, S); q = r.randint(0, 3 * S)
print(x0, a, b, c, d)
print(n, m, q)
for _ in range(q): print(r.randint(1, n * m), r.randint(1, n * m))
