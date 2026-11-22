import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(2, 4 * S + 1); print(n); m = r.choice([1, 2, 3, n])
    for v in range(2, n + 1): print(r.randint(max(1, v - m), v - 1), v)
