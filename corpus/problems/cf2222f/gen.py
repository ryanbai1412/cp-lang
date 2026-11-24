import sys, random

if sys.argv[1] == "--max":
    # a random connected graph with every weight 0..m used, so the divide and conquer splits fully
    r = random.Random(1)
    n = m = q = 300_000
    print(1)
    print(n, m, q)
    ws = list(range(m))
    r.shuffle(ws)
    out = []
    for i in range(m):
        if i < n - 1:
            u, v = r.randint(1, i + 1), i + 2
        else:
            u, v = r.randint(1, n), r.randint(1, n)
        out.append(f"{u} {v} {ws[i]}")
    print("\n".join(out))
    print(*(r.randint(1, n) for _ in range(q)))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = 4
print(t)
for _ in range(t):
    n = r.randint(1, 4 * S + 3)
    m = r.randint(1, 6 * S + 4)
    q = r.randint(1, n)
    print(n, m, q)
    max_w = r.choice([0, 1, min(m, 3), min(m, 8), m])
    for _ in range(m):
        u = r.randint(1, n)
        v = r.randint(1, n)
        w = r.randint(0, max_w)
        print(u, v, w)
    print(*(r.randint(1, n) for _ in range(q)))
