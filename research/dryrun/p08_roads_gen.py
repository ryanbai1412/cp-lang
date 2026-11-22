import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); m = random.randint(1, 2 * s)
print(n, m)
for _ in range(m): print(random.randint(1, n), random.randint(1, n))
