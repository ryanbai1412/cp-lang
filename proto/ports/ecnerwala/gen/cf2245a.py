import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); print(n, r.randint(1, n)); print(''.join(r.choice('LR') for _ in range(n)))
