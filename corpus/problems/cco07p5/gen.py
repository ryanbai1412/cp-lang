import sys, random
# seed size | --max: a hidden particle "N v p" (2 <= N <= 1e5, 0 <= v, p < N) for interactor.py
MAX = sys.argv[1] == "--max"
rng = random.Random(0 if MAX else int(sys.argv[1])); S = 10**5 if MAX else max(2, min(int(sys.argv[2]), 10**5))
n = S if MAX or rng.random() < 0.3 else rng.randint(2, S)
print(n, rng.choice([0, 1, n - 1, rng.randrange(n)]), rng.choice([0, n - 1, rng.randrange(n)]))
