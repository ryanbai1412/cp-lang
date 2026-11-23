import random, sys
random.seed(int(sys.argv[1]))
xs = [random.randint(1, random.choice([20, 1000, 100000])) for _ in range(random.randint(1, 8))]
print(len(xs)); print(*xs, sep="\n")
