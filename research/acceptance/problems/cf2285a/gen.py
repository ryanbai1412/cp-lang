import random, sys
random.seed(int(sys.argv[1]))
ns = [random.randint(1, 100 if random.random() < 0.5 else 12) for _ in range(random.randint(1, 5))]
print(len(ns)); print(*ns, sep="\n")
