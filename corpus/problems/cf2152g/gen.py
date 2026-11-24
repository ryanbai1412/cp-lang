import sys, random

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = q = 2.5e5: a 1.5e5-deep path with random bushes, half the
    # vertices with monsters, and toggles mostly near the root so each flips a long range.
    r = random.Random(2152)
    n = q = 250_000
    deep = 150_000
    par = [0, 0] + [i - 1 if i <= deep else r.randint(1, i - 1) for i in range(2, n + 1)]
    rest = list(range(2, n + 1))
    r.shuffle(rest)
    label = [1] + rest
    out = ["1", str(n), " ".join(str(int(r.random() < 0.5)) for _ in range(n))]
    for i in range(2, n + 1):
        u, v = label[par[i] - 1], label[i - 1]
        out.append(f"{u} {v}" if r.random() < 0.5 else f"{v} {u}")
    out.append(str(q))
    out += [str(label[r.randint(1, 50) - 1] if r.random() < 0.6 else r.randint(1, n)) for _ in range(q)]
    print("\n".join(out))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 3)
print(t)
for _ in range(t):
    n = r.randint(2, 3 * S)
    shape = r.randint(0, 2)
    par = [0, 0] + [(i - 1 if shape == 0 else r.randint(max(1, i - 2), i - 1) if shape == 1 else r.randint(1, i - 1)) for i in range(2, n + 1)]
    label = list(range(1, n + 1))
    rest = label[1:]
    r.shuffle(rest)
    label = [1] + rest
    p = r.random()
    print(n)
    print(*[int(r.random() < p) for _ in range(n)])
    for i in range(2, n + 1):
        u, v = label[par[i] - 1], label[i - 1]
        print(*((u, v) if r.random() < 0.5 else (v, u)))
    q = r.randint(1, 3 * S)
    print(q)
    for _ in range(q):
        print(r.randint(1, n))
