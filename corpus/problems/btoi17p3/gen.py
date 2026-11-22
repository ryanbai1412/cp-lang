import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 3); m = r.randint(1, S + 1); print(n, m, r.randint(1, m))
for i in range(2, n + 1): print(r.randint(1, i - 1), i)
for _ in range(m):
    s = r.randint(2, n); print(s, *r.sample(range(1, n + 1), s))
