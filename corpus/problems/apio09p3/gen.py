import sys, random
if sys.argv[1:] == ["--max"]:
    # N = M = 5e5: a 1 -> ... -> N path (Tarjan's deepest stack) closed into one cycle over its
    # back half, cash near 4000 everywhere and every vertex a pub
    r = random.Random(9); n = m = 500_000
    print(n, m)
    print("\n".join(f"{i} {i + 1}" for i in range(1, n)))
    print(n, n // 2)
    print("\n".join(str(r.randint(3990, 4000)) for _ in range(n)))
    print(1, n)
    print(*range(1, n + 1))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.randint(1, 2 * S + 1); print(n, m)
for _ in range(m): print(r.randint(1, n), r.randint(1, n))
for _ in range(n): print(r.randint(0, 4000))
start = r.randint(1, n); p = r.randint(1, n); print(start, p)
print(*([start] + [r.randint(1, n) for _ in range(p - 1)]))
