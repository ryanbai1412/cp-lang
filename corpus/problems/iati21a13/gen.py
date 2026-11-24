import sys, random
if sys.argv[1] == "--max":  # the largest case the bounds allow
    sys.argv[1:] = ["1", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S) if S < 1000 else S
q = r.randint(1, S) if S < 1000 else S
shape = r.randrange(4)
par = []
for v in range(2, n + 1):
    if shape == 0: p = r.randint(1, v - 1)
    elif shape == 1: p = v - 1
    elif shape == 2: p = max(1, v - r.randint(1, 3))
    else: p = r.randint(max(1, v - 10), v - 1) if r.random() < 0.9 else r.randint(1, v - 1)
    par.append(p)
perm = list(range(1, n + 1)); r.shuffle(perm[1:]) if False else None
lab = [1] + r.sample(range(2, n + 1), n - 1)
out = [str(n)]
edges = [(lab[p - 1], lab[v - 1]) for v, p in zip(range(2, n + 1), par)]
r.shuffle(edges)
for a, b in edges:
    if r.random() < 0.5: a, b = b, a
    out.append(f"{a} {b}")
dens = r.choice([0.0, 0.05, 0.5])
out.append(" ".join("1" if r.random() < dens else "0" for _ in range(n)))
out.append(str(q))
pu = r.choice([0.1, 0.5, 0.9])
for _ in range(q):
    ty = 1 if r.random() < pu else 2
    x = r.randint(1, n) if r.random() < 0.7 else r.randint(1, min(n, 5))
    m = r.random()
    k = n if m < 0.2 else (r.randint(0, min(3, n)) if m < 0.5 else r.randint(0, n))
    out.append(f"{ty} {x} {k}")
print("\n".join(out))
