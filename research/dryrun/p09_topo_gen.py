import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); m = random.randint(1, 2 * s); perm = list(range(1, n + 1)); random.shuffle(perm)
cyc = random.random() < 0.3
print(n, m)
for _ in range(m):
    a, b = sorted(random.sample(range(n), 2)) if n > 1 else (0, 0)
    if cyc and random.random() < 0.1: a, b = b, a
    print(perm[a], perm[b])
