import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
R = r.randint(1, S + 1); C = r.randint(1, S + 1)
h = r.randrange(1, R + 1, 2); w = r.randrange(1, C + 1, 2); print(R, C, h, w)
p = list(range(1, R * C + 1)); r.shuffle(p)
for i in range(R): print(*p[i * C:(i + 1) * C])
