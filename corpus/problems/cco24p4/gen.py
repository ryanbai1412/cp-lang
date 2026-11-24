import sys, random

if sys.argv[1:] == ["--max"]:
    # N = 100 rooms on a path (the largest depth and distances), labels and edges shuffled.
    r = random.Random(2024); n = 100
    perm = r.sample(range(n), n)
    e = [(perm[v - 1], perm[v]) for v in range(1, n)]; r.shuffle(e)
    print(n)
    for a, b in e: print(*((a, b) if r.random() < .5 else (b, a)))
    sys.exit()
# seed size: a tree on N = 100 rooms (size ignored); shapes: random, path, star, caterpillar, broom, binary
r = random.Random(int(sys.argv[1])); n = 100
shape = int(sys.argv[1]) % 6
par = []
for v in range(1, n):
    if shape == 0: p = r.randint(0, v - 1)
    elif shape == 1: p = v - 1
    elif shape == 2: p = 0
    elif shape == 3: p = v - 1 if v < 50 else r.randint(0, 49)
    elif shape == 4: p = v - 1 if v < 60 else 59
    else: p = (v - 1) // 2
    par.append((p, v))
perm = r.sample(range(n), n)
e = [(perm[a], perm[b]) for a, b in par]; r.shuffle(e)
print(n)
for a, b in e: print(*((a, b) if r.random() < .5 else (b, a)))
