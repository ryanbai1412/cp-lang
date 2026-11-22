import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); print(n, r.randint(1, 10**5)); print(''.join(r.choice('01') for _ in range(2 * n)))
