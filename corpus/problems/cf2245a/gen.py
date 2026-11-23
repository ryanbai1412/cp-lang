import sys, random
if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 2e5).
    r = random.Random(0); n = 200_000
    print(1); print(n, n // 2); print(''.join(r.choice('LR') for _ in range(n)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); print(n, r.randint(1, n)); print(''.join(r.choice('LR') for _ in range(n)))
