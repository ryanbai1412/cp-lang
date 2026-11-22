import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
t = random.randint(1, s)
print(t)
for _ in range(t):
    print(random.choice([random.randint(1, 10**9), random.randint(1, 1000), random.choice([1, 9, 11, 99, 999999999, 10**9])]))
