import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); m = r.randint(1, S + 1); print(n); print(m)
top = n * m if r.random() < .7 else 10 ** 6
for _ in range(n): print(*[r.randint(1, top) for _ in range(m)])
