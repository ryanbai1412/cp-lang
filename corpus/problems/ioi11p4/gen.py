import sys, random
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
