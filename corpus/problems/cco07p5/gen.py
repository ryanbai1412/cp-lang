import sys, random
# seed size: a hidden particle "N v p" (2 <= N <= 1e5, 0 <= v, p < N) for interactor.py
rng = random.Random(int(sys.argv[1])); S = max(2, min(int(sys.argv[2]), 10**5))
n = S if rng.random() < 0.3 else rng.randint(2, S)
print(n, rng.choice([0, 1, n - 1, rng.randrange(n)]), rng.choice([0, n - 1, rng.randrange(n)]))
