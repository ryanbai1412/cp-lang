import sys, random
if sys.argv[1:] == ["--max"]:
    # N = 1e5, M = 1e6 distinct corridors (a ring for connectivity plus random chords), lengths up
    # to 1e9, two exits far from chamber 0: Dijkstra relaxes every corridor
    r = random.Random(0)
    n, m = 10 ** 5, 10 ** 6
    es = {(i, i + 1) for i in range(n - 1)} | {(0, n - 1)}
    while len(es) < m:
        u, v = r.sample(range(n), 2)
        es.add((min(u, v), max(u, v)))
    out = [f"{n} {m} 2"] + [f"{u} {v} {r.randint(1, 10 ** 9)}" for u, v in r.sample(sorted(es), m)]
    out.append(f"{n // 2} {n // 2 + 1}")
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(3, S + 3); k = r.randint(1, n - 1)
es = []
for _ in range(r.randint(2, 4 * S + 2)):
    u, v = r.sample(range(n), 2); es.append((u, v))
exits = r.sample(range(1, n), k)
# guarantee an escape: chamber 0 gets two corridors straight to an exit
es += [(0, exits[0]), (0, exits[0])]
print(n, len(es), k); M = r.choice([5, 10**9])
for u, v in es: print(u, v, r.randint(1, M))
print(*exits)
