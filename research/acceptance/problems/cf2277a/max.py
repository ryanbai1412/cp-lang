import random
random.seed(1)
n = 100
cases = []
cases.append((1, list(range(n))))
cases.append((1, [0] * n))
cases.append((n, [0] * n))
cases.append((2, [i // 2 for i in range(n)]))
cases.append((3, sorted(random.randint(0, 10) for _ in range(n))))
cases.append((10, [0] * 50 + [n] * 50))
while len(cases) < 500:
    k = random.choice([1, 2, 3, random.randint(1, n)])
    hi = random.choice([n, 20, 5, 1])
    cases.append((k, [random.randint(0, hi) for _ in range(n)]))
print(len(cases))
for k, a in cases:
    print(n, k); print(*a)
