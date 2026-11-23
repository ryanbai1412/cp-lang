import random
random.seed(1)
n = q = 200000
half = [random.randint(-2, 2) for _ in range(n // 2)]
a = half + half[::-1]
print(1); print(n, 1000, q); print(*a)
print("\n".join(f"{random.randint(1, n)} {random.choice([0, 0, 1, -1])}" for _ in range(q)))
