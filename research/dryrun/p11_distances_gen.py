import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); p = list(range(1, n + 1)); random.shuffle(p)
print(n)
for i in range(1, n):
    j = random.randint(max(0, i - random.choice([1, 3, i])), i - 1)
    print(*(p[i], p[j]) if random.random() < .5 else (p[j], p[i]))
