import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); n, q = 200000, 10**6; print(n, q)
    print(*[r.choice([1, 2, 3, 1]) for _ in range(n)])
    print(*[r.randint(0, 10**9) for _ in range(q)])
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S); q = r.randint(1, 2 * S); print(n, q)
big = r.randint(0, 3) == 0
print(*[r.choice([1, 2, 3, 4, 6, 999999937, 10**9] if big else [1, 1, 2, 3, 4, 5, 6]) for _ in range(n)])
for _ in range(q): print(r.choice([r.randint(1, 10**9), r.randint(1, 60), 720, 60, 12, 2 * 3 * 5 * 7 * 11 * 13]))
