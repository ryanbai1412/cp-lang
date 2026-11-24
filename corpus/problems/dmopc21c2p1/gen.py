import sys, random
if sys.argv[1:] == ["--max"]:
    # N = 1e6 distinct-ish powers up to 1e6 with H, P near 1e6: the scan tries every A_i
    r = random.Random(0)
    n = 10 ** 6
    print(n, 10 ** 6, 10 ** 6 - 1)
    print(*[r.randint(0, 10 ** 6) for _ in range(n)])
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 5 * 10**5
n = 10**6 if big else r.randint(1, 2 * S); M = 10**6 if big else r.choice([5, 10**6]); print(n, r.randint(1, M), r.randint(1, M))
print(*[r.randint(0, M) for _ in range(n)])
