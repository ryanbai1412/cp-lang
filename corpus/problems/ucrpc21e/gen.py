import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
T = 10**9 if big else r.randint(1, r.choice([10, 100, 10**9]))
def windows():
    k = S if big else r.randint(0, min(S, 100000, T))
    ends = sorted(r.sample(range(1, T + 1), min(2 * k, T))) if 2 * k <= T else sorted(r.sample(range(1, T + 1), k) * 2)
    out = [(ends[i], ends[i + 1]) for i in range(0, len(ends) - 1, 2)]
    return [(a, b, r.randint(0, 1)) for a, b in out]
u, l = windows(), windows()
print(T, len(u), len(l))
for w in u + l: print(*w)
