import sys, random
if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 2e5): a random split of
    # the permutation of 1..2n into the two rows.
    r = random.Random(0); n = 200000; print(1); print(n)
    p = list(range(1, 2 * n + 1)); r.shuffle(p); print(*p[:n]); print(*p[n:])
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 3); print(n)
    p = list(range(1, 2 * n + 1)); r.shuffle(p)
    print(*p[:n])
    print(*p[n:])
