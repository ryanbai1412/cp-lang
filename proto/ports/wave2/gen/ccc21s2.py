import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); m = r.randint(1, S + 1); k = r.randint(1, 3 * S); print(n); print(m); print(k)
for _ in range(k):
    if r.random() < .5: print('R', r.randint(1, n))
    else: print('C', r.randint(1, m))
