import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); p = r.choice([0.1, 0.5, 0.9]); print(n); print(''.join('1' if r.random() < p else '0' for _ in range(n)))
