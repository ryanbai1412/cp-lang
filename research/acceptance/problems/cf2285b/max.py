import random
random.seed(1)
print(2)
for n in (200000, 100000):
    print(n, random.randint(0, n)); print("".join(random.choice("01") for _ in range(n)))
