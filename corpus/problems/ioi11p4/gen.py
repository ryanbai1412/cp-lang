import sys, random
if sys.argv[1:] == ["--max"]:
    # N = 1e5, M = 1e6 distinct corridors, lengths up to 1e9. A ladder (i to i + 1 and i + 2)
    # climbs from chamber 0 to the exits N - 2 and N - 1, so every chamber keeps a way out
    # whatever corridor is blocked; random chords on top and 9998 more exits keep the answer
    # under 1e9 (the grader returns an int) while Dijkstra relaxes every corridor.
    r = random.Random(0)
    n, m = 10 ** 5, 10 ** 6
    es = {(i, i + 1) for i in range(n - 1)} | {(i, i + 2) for i in range(n - 2)}
    while len(es) < m:
        u, v = r.sample(range(n), 2)
        es.add((min(u, v), max(u, v)))
    exits = [n - 2, n - 1] + r.sample(range(1, n - 2), 9998)
    out = [f"{n} {m} {len(exits)}"] + [f"{u} {v} {r.randint(1, 10 ** 9)}" for u, v in r.sample(sorted(es), m)]
    out.append(" ".join(map(str, exits)))
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
