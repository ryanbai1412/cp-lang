import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, max(1, s // 3)); q = random.randint(1, s)
print(n, q)
for _ in range(n): print(''.join(random.choice('.*') for _ in range(n)))
for _ in range(q):
    a = random.randint(1, n); c = random.randint(a, n); b = random.randint(1, n); d = random.randint(b, n); print(a, b, c, d)
