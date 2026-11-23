import random, sys
random.seed(int(sys.argv[1]))
M = 1 << 25
pick = random.choice([lambda: random.randint(1, M - 1), lambda: random.randint(1, 15), lambda: M - 1, lambda: 1 << random.randrange(25)])
def dense(n):  # values from a span of dimension <= 3: many short zero sums, repeats, zeros
    basis = [pick() for _ in range(random.randint(1, 3 if n <= 5 else 2))]
    a = []
    for _ in range(n):
        x = 0
        for b in basis:
            if random.random() < 0.6: x ^= b
        a.append(x if x or random.random() < 0.2 else basis[0])
    return a
def sparse(n):  # n - 1 independent values and one planted zero sum: long answers or -1
    a = [1 << i for i in random.sample(range(25), n - 1)] if random.random() < 0.5 else [random.randint(1, M - 1) for _ in range(n - 1)]
    x = 0
    for v in random.sample(a, random.randint(1, n - 1)): x ^= v
    a.insert(random.randrange(n), x if random.random() < 0.8 else pick())
    return a
t = random.randint(1, 10); print(t)
slow = 2
for _ in range(t):
    if slow and random.random() < 0.4:
        slow -= 1; a = sparse(random.randint(2, 5))
    else:
        a = dense(random.choice([1, 2, 3, 4, 5, 6, 7]))
    print(len(a)); print(*a)
