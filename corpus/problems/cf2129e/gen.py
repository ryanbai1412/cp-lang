#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at every sum (n = m = q = 1.5e5): random edges plus hubs, so arc windows are
    # uneven, and wide random queries to keep Mo's pointers moving.
    r = random.Random(2129)
    n = m = q = 150_000
    edges = set()
    hubs = [r.randint(1, n) for _ in range(30)]
    while len(edges) < m:
        u = r.choice(hubs) if len(edges) < m // 3 else r.randint(1, n)
        v = r.randint(1, n)
        if u != v: edges.add((min(u, v), max(u, v)))
    out = ["1", f"{n} {m}"]
    out += [f"{u} {v}" if r.random() < 0.5 else f"{v} {u}" for u, v in edges]
    out.append(str(q))
    for _ in range(q):
        l = r.randint(1, n // 2)
        rr = r.randint(l, n)
        out.append(f"{l} {rr} {r.randint(1, rr - l + 1)}")
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 25)
    all_e = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    r.shuffle(all_e)
    m = r.randint(1, min(len(all_e), 2 * n))
    print(n, m)
    for u, v in all_e[:m]:
        print(u, v)
    q = r.randint(1, 8 if size == 0 else 30)
    print(q)
    for _ in range(q):
        l = r.randint(1, n)
        rr = r.randint(l, n)
        k = r.randint(1, rr - l + 1)
        print(l, rr, k)
